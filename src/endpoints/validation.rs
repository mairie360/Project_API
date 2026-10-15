//! Input validation shared by every request body and query string of the API.
//!
//! A request view implements [`Validate`] and the handler extracts it with [`ValidatedJson`] or
//! [`ValidatedQuery`] instead of `web::Json` / `web::Query`: an invalid value is rejected with a
//! `400 Bad Request` (plain-text body naming the field) before the handler runs, so it never
//! reaches Postgres (where an over-long value or a NUL byte used to end in a `500`).
//!
//! `<` and `>` are accepted everywhere (MAIR-426): "budget > 10 000 €" or "->" are ordinary text.
//! The API serves JSON with `nosniff`, which no browser renders as HTML; escaping is the fronts'
//! job (React escapes text, none of them uses `dangerouslySetInnerHTML`).

use std::fmt;
use std::future::Future;
use std::pin::Pin;

use actix_web::{dev::Payload, web, FromRequest, HttpRequest};
use chrono::{DateTime, NaiveDate, Utc};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer};

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

/// A short label (project or task name, custom field label): not blank, at most `max` characters,
/// no control character.
///
/// # Errors
///
/// Returns a [`ValidationError`] naming `field` when one of the rules is broken.
pub fn check_label(field: &str, value: &str, max: usize) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        return Err(ValidationError::new(field, "must not be empty"));
    }
    check_length(field, value, max)?;
    check_no_control(field, value)
}

/// A free-text description: may be empty, at most `max` characters, line breaks and tabs
/// allowed, no other control character.
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
    Ok(())
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

/// A free JSON value (custom field option): no string or key may contain a control character
/// other than line breaks and tabs (`jsonb` rejects `\u0000`).
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

/// A comma-separated list whose every item is one of `allowed` (case-sensitive, no blank item).
///
/// # Errors
///
/// Returns a [`ValidationError`] naming `field` and listing the allowed values otherwise.
pub fn check_list(field: &str, value: &str, allowed: &[&str]) -> Result<(), ValidationError> {
    if value.split(',').all(|item| allowed.contains(&item)) {
        return Ok(());
    }
    Err(ValidationError::new(
        field,
        &format!("must be a comma-separated list of {}", allowed.join(", ")),
    ))
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

/// Parses a task due date: an RFC 3339 date-time (`2026-11-05T09:00:00+01:00`), or a date alone
/// (`2026-11-05`, what a date picker sends), read as midnight UTC of that day (MAIR-559).
///
/// # Errors
///
/// Returns a [`ValidationError`] naming `field` when the value is neither.
pub fn parse_due_date(field: &str, value: &str) -> Result<DateTime<Utc>, ValidationError> {
    if let Ok(date_time) = DateTime::parse_from_rfc3339(value) {
        return Ok(date_time.with_timezone(&Utc));
    }
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .ok()
        .and_then(|date| date.and_hms_opt(0, 0, 0))
        .map(|midnight| midnight.and_utc())
        .ok_or_else(|| {
            ValidationError::new(
                field,
                "must be a date (`YYYY-MM-DD`) or an RFC 3339 date-time",
            )
        })
}

/// `deserialize_with` of an optional `due_date` body field, through [`parse_due_date`]: absent or
/// `null` give `None`. Use it with `#[serde(default)]`.
///
/// # Errors
///
/// Fails the deserialization (hence a `400`) when the value is not a string or not a due date.
pub fn deserialize_optional_due_date<'de, D>(
    deserializer: D,
) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer)?
        .map(|value| parse_due_date("due_date", &value).map_err(serde::de::Error::custom))
        .transpose()
}

/// Cap of a task status or project template name (`VARCHAR(100)`, MAIR-566).
pub const MAX_STATUS_NAME_LENGTH: usize = 100;
/// Most statuses a template may define.
pub const MAX_TEMPLATE_STATUSES: usize = 30;

/// Checks a status colour: `#` followed by 6 hexadecimal digits (`#2563EB`).
///
/// # Errors
///
/// Returns a [`ValidationError`] naming `field` otherwise.
pub fn check_color(field: &str, value: &str) -> Result<(), ValidationError> {
    let digits = value.strip_prefix('#').unwrap_or_default();
    if digits.len() == 6 && digits.chars().all(|c| c.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(ValidationError::new(
            field,
            "must be `#` followed by 6 hexadecimal digits",
        ))
    }
}

/// Checks the statuses of a template (MAIR-566): 1 to [`MAX_TEMPLATE_STATUSES`] statuses, names
/// unique whatever the case, and every category in `required` covered.
///
/// # Errors
///
/// Returns a [`ValidationError`] naming `field`.
pub fn check_status_list<C: PartialEq + fmt::Debug>(
    field: &str,
    statuses: &[(&str, C)],
    required: &[C],
) -> Result<(), ValidationError> {
    if statuses.is_empty() || statuses.len() > MAX_TEMPLATE_STATUSES {
        return Err(ValidationError::new(
            field,
            &format!("must hold 1 to {MAX_TEMPLATE_STATUSES} statuses"),
        ));
    }
    let mut names = std::collections::HashSet::new();
    for (name, _) in statuses {
        if !names.insert(name.trim().to_lowercase()) {
            return Err(ValidationError::new(
                field,
                "two statuses have the same name",
            ));
        }
    }
    for category in required {
        if !statuses.iter().any(|(_, c)| c == category) {
            return Err(ValidationError::new(
                field,
                &format!("must hold a status of each category, none is {category:?}"),
            ));
        }
    }
    Ok(())
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
    fn label_rejects_blank_long_and_control() {
        assert!(check_label("name", "Réfection de la place", MAX_TITLE_LENGTH).is_ok());
        assert!(check_label("name", "  ", MAX_TITLE_LENGTH).is_err());
        assert!(check_label("name", &"a".repeat(256), MAX_TITLE_LENGTH).is_err());
        assert!(check_label("name", "Place\0du marché", MAX_TITLE_LENGTH).is_err());
        // Angle brackets are ordinary text (MAIR-426).
        assert!(check_label("name", "Budget > 10 000 € -> <3", MAX_TITLE_LENGTH).is_ok());
    }

    #[test]
    fn list_accepts_only_allowed_items() {
        let allowed = ["Low", "High"];
        assert!(check_list("priority", "Low", &allowed).is_ok());
        assert!(check_list("priority", "Low,High", &allowed).is_ok());
        assert!(check_list("priority", "", &allowed).is_err());
        assert!(check_list("priority", "Low,", &allowed).is_err());
        assert!(check_list("priority", "low", &allowed).is_err());
        assert!(check_list("priority", "Low, High", &allowed).is_err());
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
        assert!(check_description("description", "<b>1 < 2</b>", MAX_DESCRIPTION_LENGTH).is_ok());
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
        let markup = serde_json::json!({ "a > b": "<3" });
        assert!(check_json("changes", &markup).is_ok());
    }

    #[test]
    fn optional_skips_absent_values() {
        assert!(check_optional(None, |v| check_label("name", v, MAX_TITLE_LENGTH)).is_ok());
        assert!(check_optional(Some(" "), |v| check_label("name", v, MAX_TITLE_LENGTH)).is_err());
    }

    #[test]
    fn mair_566_status_colors_and_lists() {
        assert!(check_color("color", "#2563EB").is_ok());
        assert!(check_color("color", "#2563eb").is_ok());
        for invalid in ["2563EB", "#2563E", "#2563EBA", "#GGGGGG", "", "blue"] {
            assert!(check_color("color", invalid).is_err(), "{invalid}");
        }
        let all = [1, 2, 3];
        assert!(check_status_list("statuses", &[("A", 1), ("B", 2), ("C", 3)], &all).is_ok());
        assert!(check_status_list::<i32>("statuses", &[], &all).is_err());
        let duplicate = check_status_list(
            "statuses",
            &[("À faire", 1), (" à faire ", 2), ("C", 3)],
            &all,
        );
        assert!(duplicate.unwrap_err().to_string().contains("same name"));
        let missing = check_status_list("statuses", &[("A", 1), ("B", 2)], &all);
        assert!(missing.unwrap_err().to_string().contains("none is 3"));
        let many: Vec<(&str, i32)> = (0..=MAX_TEMPLATE_STATUSES)
            .map(|i| (["A", "B", "C"][i % 3], 1))
            .collect();
        assert!(check_status_list("statuses", &many, &[1]).is_err());
    }

    #[test]
    fn mair_559_due_date_accepts_a_date_alone() {
        let midnight = parse_due_date("due_date", "2026-11-05").unwrap();
        assert_eq!(midnight.to_rfc3339(), "2026-11-05T00:00:00+00:00");
        let offset = parse_due_date("due_date", "2026-11-05T09:30:00+01:00").unwrap();
        assert_eq!(offset.to_rfc3339(), "2026-11-05T08:30:00+00:00");
        for invalid in ["", "05/11/2026", "2026-13-01", "2026-02-30", "demain"] {
            let error = parse_due_date("due_date", invalid).unwrap_err();
            assert!(error.to_string().contains("`due_date`"), "{invalid}");
        }
    }

    #[test]
    fn mair_559_due_date_field_absent_null_or_date() {
        #[derive(Deserialize)]
        struct Body {
            #[serde(default, deserialize_with = "deserialize_optional_due_date")]
            due_date: Option<DateTime<Utc>>,
        }
        let read = |json: &str| serde_json::from_str::<Body>(json).map(|body| body.due_date);
        assert_eq!(read("{}").unwrap(), None);
        assert_eq!(read(r#"{"due_date":null}"#).unwrap(), None);
        assert_eq!(
            read(r#"{"due_date":"2026-11-05"}"#).unwrap(),
            Some(parse_due_date("due_date", "2026-11-05T00:00:00Z").unwrap())
        );
        assert!(read(r#"{"due_date":"05/11/2026"}"#).is_err());
        assert!(read(r#"{"due_date":20261105}"#).is_err());
    }
}
