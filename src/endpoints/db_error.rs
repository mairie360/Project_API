//! One way to turn a database error into an HTTP answer (MAIR-421).
//!
//! Handlers never write `.map_err(|_| MyError::DatabaseError)`: that drops the cause and turns a
//! constraint violation or a missing row into a `500` (or a `500` into a `400`). They call
//! [`classify_db_error`] instead, which logs the error with the operation it happened in and says
//! what kind of failure it is; the handler's error enum maps each [`DbFailure`] to its own variant
//! (and documents the matching status in its `#[utoipa::path]`).

use std::fmt::Display;

use mairie360_api_lib::database::error::DbError;
use mairie360_api_lib::error::ApiLibError;

/// What a failed query means for the client.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbFailure {
    /// No row matched: usually `404`.
    NotFound,
    /// A unique constraint refused the write: usually `409`.
    Conflict,
    /// A foreign key points to a row that does not exist: usually `400` (or `404` when the
    /// reference comes from the URL).
    InvalidReference,
    /// A check constraint refused the value (SQLSTATE `23514`): usually `400`.
    CheckViolation,
    /// Anything else (connection lost, SQL error, mapping error): `500`, logged at `error`.
    Internal,
}

/// Logs `error` with the `operation` it happened in (e.g. `"projects/post"`) and classifies it.
///
/// Client-side failures are logged at `info`, everything else at `error` with its full cause, so a
/// `500` in production always has a log line saying why.
pub fn classify_db_error(operation: &str, error: &ApiLibError) -> DbFailure {
    let failure = match error {
        ApiLibError::Database(DbError::NotFound) => DbFailure::NotFound,
        ApiLibError::Database(DbError::UniqueViolation(_)) => DbFailure::Conflict,
        ApiLibError::Database(DbError::ForeignKeyViolation(_)) => DbFailure::InvalidReference,
        // The lib has no variant for a check violation: it keeps the SQLSTATE in the text of
        // `DbError::Sqlx`, never the value (MAIR-290).
        ApiLibError::Database(DbError::Sqlx(_)) if error.to_string().contains("SQLSTATE 23514") => {
            DbFailure::CheckViolation
        }
        _ => DbFailure::Internal,
    };
    if failure == DbFailure::Internal {
        log_db_error(operation, error);
    } else {
        tracing::info!(operation, ?failure, %error, "database refused the request");
    }
    failure
}

/// Name of the foreign key a write broke, `None` for any other error: lets a handler tell which
/// reference of its body is unknown when it writes several.
pub fn violated_foreign_key(error: &ApiLibError) -> Option<&str> {
    match error {
        ApiLibError::Database(DbError::ForeignKeyViolation(constraint)) => Some(constraint),
        _ => None,
    }
}

/// Logs a database error at `error` before the handler turns it into an opaque `500`.
pub fn log_db_error(operation: &str, error: &impl Display) {
    tracing::error!(operation, %error, "database error");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn classify(error: DbError) -> DbFailure {
        classify_db_error("test", &ApiLibError::Database(error))
    }

    #[test]
    fn mair_1196_names_the_broken_foreign_key_only() {
        let broken = ApiLibError::Database(DbError::ForeignKeyViolation(
            "fk_projects_responsible".into(),
        ));
        assert_eq!(
            violated_foreign_key(&broken),
            Some("fk_projects_responsible")
        );
        assert_eq!(
            violated_foreign_key(&ApiLibError::Database(DbError::NotFound)),
            None
        );
    }

    #[test]
    fn maps_each_database_error_to_its_failure() {
        assert_eq!(classify(DbError::NotFound), DbFailure::NotFound);
        assert_eq!(
            classify(DbError::UniqueViolation("users_email_key".into())),
            DbFailure::Conflict
        );
        assert_eq!(
            classify(DbError::ForeignKeyViolation("fk_owner".into())),
            DbFailure::InvalidReference
        );
        assert_eq!(
            classify(DbError::Internal("pool closed".into())),
            DbFailure::Internal
        );
        assert_eq!(
            classify(DbError::MappingError("missing field".into())),
            DbFailure::Internal
        );
    }

    #[test]
    fn non_database_errors_are_internal() {
        let error =
            ApiLibError::Serialization(serde_json::from_str::<u8>("x").expect_err("invalid JSON"));
        assert_eq!(classify_db_error("test", &error), DbFailure::Internal);
    }
}
