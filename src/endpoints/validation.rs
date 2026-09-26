//! Input validation shared by every request body and query string of the API.
//!
//! A request view implements [`Validate`] and the handler extracts it with [`ValidatedJson`] or
//! [`ValidatedQuery`] instead of `web::Json` / `web::Query`: an invalid value is rejected with a
//! `400 Bad Request` (plain-text body naming the field) before the handler runs, so it never
//! reaches Postgres (where an over-long value or a NUL byte used to end in a `500`) nor comes back
//! unescaped in a JSON response.

use std::fmt;
use std::future::Future;
use std::pin::Pin;

use actix_web::{dev::Payload, web, FromRequest, HttpRequest};
use serde::de::DeserializeOwned;

/// `projects.title` and `tasks.title` are `VARCHAR(255)`; also the cap of custom field and history
/// labels (stored in JSONB).
pub const MAX_TITLE_LENGTH: usize = 255;
/// Project and task descriptions (`TEXT`, capped to keep the payloads reasonable).
pub const MAX_DESCRIPTION_LENGTH: usize = 5000;
/// Short machine identifiers sent by the client (history `action`).
pub const MAX_IDENTIFIER_LENGTH: usize = 64;

/// Why a request value was rejected; its text is the body of the `400` response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError(String);

impl ValidationError {
    fn new(field: &str, reason: &str) -> Self {
        Self(format!("Invalid `{field}`: {reason}"))
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Implemented by every request view extracted with [`ValidatedJson`] or [`ValidatedQuery`].
pub trait Validate {
    /// # Errors
    ///
    /// Returns the first field that does not satisfy its constraints.
    fn validate(&self) -> Result<(), ValidationError>;
}

fn check_length(field: &str, value: &str, max: usize) -> Result<(), ValidationError> {
    if value.chars().count() > max {
        return Err(ValidationError::new(
            field,
            &format!("must be at most {max} characters"),
        ));
    }
    Ok(())
}

fn check_no_control(field: &str, value: &str) -> Result<(), ValidationError> {
    if value.chars().any(char::is_control) {
        return Err(ValidationError::new(
            field,
            "must not contain control characters",
        ));
    }
    Ok(())
}

fn check_no_markup(field: &str, value: &str) -> Result<(), ValidationError> {
    if value.contains(['<', '>']) {
        return Err(ValidationError::new(field, "must not contain `<` or `>`"));
    }
    Ok(())
}

/// A short label displayed as-is by the fronts (person name, role or group name): not blank, at
/// most `max` characters, no control character and no `<` / `>`.
///
/// # Errors
///
/// Returns a [`ValidationError`] naming `field` when one of the rules is broken.
pub fn check_label(field: &str, value: &str, max: usize) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        return Err(ValidationError::new(field, "must not be empty"));
    }
    check_length(field, value, max)?;
    check_no_control(field, value)?;
    check_no_markup(field, value)
}

/// A free-text description: may be empty, at most `max` characters, line breaks and tabs
/// allowed, no other control character and no `<` / `>`.
///
/// # Errors
///
/// Returns a [`ValidationError`] naming `field` when one of the rules is broken.
pub fn check_description(field: &str, value: &str, max: usize) -> Result<(), ValidationError> {
    check_length(field, value, max)?;
    if value
        .chars()
        .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
    {
        return Err(ValidationError::new(
            field,
            "must not contain control characters other than line breaks and tabs",
        ));
    }
    check_no_markup(field, value)
}

/// An opaque value only compared or stored as text (token, credential, `device_info`, search
/// filter): at most `max` characters and no control character (Postgres rejects NUL bytes).
///
/// # Errors
///
/// Returns a [`ValidationError`] naming `field` when one of the rules is broken.
pub fn check_opaque(field: &str, value: &str, max: usize) -> Result<(), ValidationError> {
    check_length(field, value, max)?;
    check_no_control(field, value)
}

/// A free JSON value (custom field option, history `changes`): no string or key may contain a
/// control character other than line breaks and tabs (`jsonb` rejects `\u0000`) nor `<` / `>`.
///
/// # Errors
///
/// Returns a [`ValidationError`] naming `field` when one of the strings breaks the rules.
pub fn check_json(field: &str, value: &serde_json::Value) -> Result<(), ValidationError> {
    match value {
        serde_json::Value::String(text) => check_description(field, text, MAX_DESCRIPTION_LENGTH),
        serde_json::Value::Array(items) => {
            items.iter().try_for_each(|item| check_json(field, item))
        }
        serde_json::Value::Object(map) => map.iter().try_for_each(|(key, item)| {
            check_description(field, key, MAX_TITLE_LENGTH)?;
            check_json(field, item)
        }),
        _ => Ok(()),
    }
}

/// Runs `check` on `value` when it is present.
///
/// # Errors
///
/// Returns the error of `check`.
pub fn check_optional<F>(value: Option<&str>, check: F) -> Result<(), ValidationError>
where
    F: FnOnce(&str) -> Result<(), ValidationError>,
{
    value.map_or(Ok(()), check)
}

fn bad_request(error: &ValidationError) -> actix_web::Error {
    actix_web::error::ErrorBadRequest(error.to_string())
}

/// `web::Json<T>` followed by [`Validate::validate`]: answers `400` when either fails.
pub struct ValidatedJson<T>(pub T);

impl<T> ValidatedJson<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> FromRequest for ValidatedJson<T>
where
    T: DeserializeOwned + Validate + 'static,
{
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let json = web::Json::<T>::from_request(req, payload);
        Box::pin(async move {
            let value = json.await?.into_inner();
            value.validate().map_err(|e| bad_request(&e))?;
            Ok(Self(value))
        })
    }
}

/// `web::Query<T>` followed by [`Validate::validate`]: answers `400` when either fails.
pub struct ValidatedQuery<T>(pub T);

impl<T> ValidatedQuery<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> FromRequest for ValidatedQuery<T>
where
    T: DeserializeOwned + Validate + 'static,
{
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let query = web::Query::<T>::from_query(req.query_string());
        Box::pin(async move {
            let value = query?.into_inner();
            value.validate().map_err(|e| bad_request(&e))?;
            Ok(Self(value))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_rejects_blank_long_control_and_markup() {
        assert!(check_label("name", "Réfection de la place", MAX_TITLE_LENGTH).is_ok());
        assert!(check_label("name", "  ", MAX_TITLE_LENGTH).is_err());
        assert!(check_label("name", &"a".repeat(256), MAX_TITLE_LENGTH).is_err());
        assert!(check_label("name", "Place\0du marché", MAX_TITLE_LENGTH).is_err());
        assert!(check_label("name", "<script>alert(1);</script>", MAX_TITLE_LENGTH).is_err());
    }

    #[test]
    fn label_counts_characters_not_bytes() {
        assert!(check_label("name", &"é".repeat(255), MAX_TITLE_LENGTH).is_ok());
    }

    #[test]
    fn description_allows_line_breaks_only() {
        assert!(check_description("description", "", MAX_DESCRIPTION_LENGTH).is_ok());
        assert!(check_description("description", "a\nb\tc", MAX_DESCRIPTION_LENGTH).is_ok());
        assert!(check_description("description", "a\0b", MAX_DESCRIPTION_LENGTH).is_err());
        assert!(check_description("description", "<b>", MAX_DESCRIPTION_LENGTH).is_err());
    }

    #[test]
    fn opaque_rejects_nul() {
        assert!(check_opaque("action", "task_updated", MAX_IDENTIFIER_LENGTH).is_ok());
        assert!(check_opaque("action", "task\0", MAX_IDENTIFIER_LENGTH).is_err());
    }

    #[test]
    fn json_checks_nested_strings_and_keys() {
        let ok = serde_json::json!({ "budget": { "from": 12000, "to": [1, "Oui"] } });
        assert!(check_json("changes", &ok).is_ok());
        let nul = serde_json::json!({ "budget": ["a\0"] });
        assert!(check_json("changes", &nul).is_err());
        let markup = serde_json::json!({ "<script>": 1 });
        assert!(check_json("changes", &markup).is_err());
    }

    #[test]
    fn optional_skips_absent_values() {
        assert!(check_optional(None, |v| check_label("name", v, MAX_TITLE_LENGTH)).is_ok());
        assert!(check_optional(Some(" "), |v| check_label("name", v, MAX_TITLE_LENGTH)).is_err());
    }
}
