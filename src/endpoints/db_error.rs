use std::fmt::Display;

use mairie360_api_lib::database::error::DbError;
use mairie360_api_lib::error::ApiLibError;

/// Logs a database error before the handler turns it into an opaque `500`, so the cause is not lost.
pub fn log_db_error(operation: &str, error: &impl Display) {
    tracing::error!(operation, %error, "database error");
}

/// What a failed query means for the client. Every handler maps a database error through
/// [`classify`] instead of matching `DbError` by hand, so a constraint violation never ends up as
/// a `500` and an unexpected failure is never answered without a log line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbFailure {
    /// Unique constraint: the row already exists (`409`).
    Conflict,
    /// Foreign key: an id of the request does not match any row (`400` or `404`, per handler).
    InvalidReference,
    /// The query expected a row and got none (`404`).
    NotFound,
    /// Anything else: logged at `error` level, answered `500`.
    Internal,
}

/// Classifies `error` and logs it under `operation`: client-caused failures at `warn` level
/// (expected, but worth tracing), the others at `error` level.
pub fn classify(operation: &str, error: &ApiLibError) -> DbFailure {
    let failure = match error {
        ApiLibError::Database(DbError::UniqueViolation(_)) => DbFailure::Conflict,
        ApiLibError::Database(DbError::ForeignKeyViolation(_)) => DbFailure::InvalidReference,
        ApiLibError::Database(DbError::NotFound) => DbFailure::NotFound,
        _ => DbFailure::Internal,
    };
    if failure == DbFailure::Internal {
        log_db_error(operation, error);
    } else {
        tracing::warn!(operation, ?failure, %error, "database constraint refused the request");
    }
    failure
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_maps_each_database_error() {
        let cases = [
            (DbError::UniqueViolation("dup".into()), DbFailure::Conflict),
            (
                DbError::ForeignKeyViolation("fk".into()),
                DbFailure::InvalidReference,
            ),
            (DbError::NotFound, DbFailure::NotFound),
            (DbError::Internal("boom".into()), DbFailure::Internal),
            (DbError::MappingError("json".into()), DbFailure::Internal),
        ];
        for (error, expected) in cases {
            assert_eq!(classify("test", &ApiLibError::Database(error)), expected);
        }
        let serialization = serde_json::from_str::<u8>("x").unwrap_err();
        assert_eq!(
            classify("test", &ApiLibError::Serialization(serialization)),
            DbFailure::Internal
        );
    }
}
