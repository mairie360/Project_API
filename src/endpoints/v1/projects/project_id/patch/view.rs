use utoipa::ToSchema;

use crate::endpoints::v1::projects::get::view::ProjectStatus;
use crate::endpoints::v1::projects::project_id::get::view::TaskPriority;
use crate::endpoints::validation::{
    check_description, check_label, check_optional, check_required, Validate, ValidationError,
    MAX_DESCRIPTION_LENGTH, MAX_TITLE_LENGTH,
};

/// Partial update of a project: only the given fields are updated.
#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct UpdateProjectView {
    /// New name. Absent or `null` to leave it unchanged; cannot be only spaces.
    #[schema(
        min_length = 1,
        max_length = 255,
        example = "Renovation of the market square"
    )]
    pub name: Option<String>,
    /// New description. Absent or `null` to leave it unchanged.
    #[schema(max_length = 5000, example = "2026 road works, phase 2")]
    pub description: Option<String>,
    /// New status. `Error` is refused: it is only a fallback on read.
    pub status: Option<ProjectStatus>,
    /// New priority. Absent or `null` to keep it; `Urgent` is stored `High`, `Error` is refused.
    #[serde(default)]
    pub priority: Option<TaskPriority>,
    /// Group linked to the project: absent to keep it, an id to replace it. `null` is refused
    /// (`400`): a project always keeps a group. One of the caller's groups, any group for Admin
    /// and Maire (`GET /api/v1/project-groups/`).
    #[serde(default, deserialize_with = "present")]
    #[schema(value_type = Option<u64>, nullable = false, example = 3)]
    pub group_id: Option<Option<u64>>,
    /// Budget voted by the council, in cents of euro (MAIR-1085: `1250000` is 12 500,00 €):
    /// absent to keep it, `null` to clear it, an integer from 0 to replace it. A negative amount is
    /// refused (`400`).
    #[serde(default, deserialize_with = "present")]
    #[schema(value_type = Option<i64>, nullable, minimum = 0, example = 1250000)]
    pub budget_voted: Option<Option<i64>>,
    /// Budget already committed, in cents of euro (MAIR-1085): absent to keep it, `null` to clear
    /// it, an integer from 0 to replace it. A negative amount is refused (`400`).
    #[serde(default, deserialize_with = "present")]
    #[schema(value_type = Option<i64>, nullable, minimum = 0, example = 480000)]
    pub budget_committed: Option<Option<i64>>,
    /// Responsible of the project: absent to keep it, `null` to clear it, a user id to replace it.
    /// The new responsible becomes a `manager` member (added, or promoted when already a member);
    /// the former one keeps their membership and role. An unknown user is refused (`400`).
    #[serde(default, deserialize_with = "present")]
    #[schema(value_type = Option<u64>, nullable, example = 7)]
    pub responsible_id: Option<Option<u64>>,
}

/// Refuses a negative amount of cents.
fn check_amount(field: &str, amount: Option<Option<i64>>) -> Result<(), ValidationError> {
    match amount.flatten() {
        Some(cents) if cents < 0 => Err(ValidationError::invalid(field, "must not be negative")),
        _ => Ok(()),
    }
}

/// Tells a field given `null` (`Some(None)`) from an absent one (`None`, through `default`).
fn present<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    serde::Deserialize::deserialize(deserializer).map(Some)
}

impl Validate for UpdateProjectView {
    fn validate(&self) -> Result<(), ValidationError> {
        check_optional(self.name.as_deref(), |name| {
            check_label("name", name.trim(), MAX_TITLE_LENGTH)
        })?;
        if let Some(group_id) = self.group_id {
            check_required("group_id", group_id)?;
        }
        check_amount("budget_voted", self.budget_voted)?;
        check_amount("budget_committed", self.budget_committed)?;
        check_optional(self.description.as_deref(), |description| {
            check_description("description", description, MAX_DESCRIPTION_LENGTH)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(body: serde_json::Value) -> UpdateProjectView {
        serde_json::from_value(body).unwrap()
    }

    #[test]
    fn mair_683_removing_the_group_is_refused_but_changing_or_keeping_it_is_not() {
        assert_eq!(
            parse(serde_json::json!({ "group_id": null }))
                .validate()
                .unwrap_err()
                .to_string(),
            "Invalid `group_id`: is required"
        );
        assert!(parse(serde_json::json!({ "group_id": 5 }))
            .validate()
            .is_ok());
        assert!(parse(serde_json::json!({ "name": "Square" }))
            .validate()
            .is_ok());
    }

    #[test]
    fn mair_1085_the_budget_is_kept_cleared_or_replaced_never_negative() {
        let absent = parse(serde_json::json!({ "name": "Square" }));
        assert_eq!((absent.budget_voted, absent.budget_committed), (None, None));
        let cleared = parse(serde_json::json!({ "budget_voted": null, "budget_committed": 0 }));
        assert_eq!(cleared.budget_voted, Some(None));
        assert_eq!(cleared.budget_committed, Some(Some(0)));
        assert!(cleared.validate().is_ok());
        for (field, body) in [
            ("budget_voted", serde_json::json!({ "budget_voted": -1 })),
            (
                "budget_committed",
                serde_json::json!({ "budget_committed": -100 }),
            ),
        ] {
            assert_eq!(
                parse(body).validate().unwrap_err().to_string(),
                format!("Invalid `{field}`: must not be negative")
            );
        }
        for invalid in [
            serde_json::json!({ "budget_voted": 12.5 }),
            serde_json::json!({ "budget_voted": "1000" }),
            serde_json::json!({ "budget_committed": 9_223_372_036_854_775_808_u64 }),
        ] {
            assert!(serde_json::from_value::<UpdateProjectView>(invalid).is_err());
        }
    }
}
