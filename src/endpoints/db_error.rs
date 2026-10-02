use std::fmt::Display;

/// Logs a database error before the handler turns it into an opaque `500`, so the cause is not lost.
pub fn log_db_error(operation: &str, error: &impl Display) {
    tracing::error!(operation, %error, "database error");
}
