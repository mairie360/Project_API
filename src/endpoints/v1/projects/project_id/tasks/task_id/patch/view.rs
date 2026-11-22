use crate::endpoints::validation::{
    check_dates_in_order, check_description, check_json, check_label, check_optional,
    deserialize_optional_due_date, deserialize_present_start_date, Validate, ValidationError,
    MAX_DESCRIPTION_LENGTH, MAX_TITLE_LENGTH,
};
use crate::{
    database::tasks::get_project_tasks::view::DynamicTaskField,
    endpoints::v1::projects::project_id::get::view::{TaskPriority, TaskStatus},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer};
use utoipa::ToSchema;

/// Tells an absent field (`None`) from an explicit `null` (`Some(None)`).
fn deserialize_present<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

/// Partial update of a task: only the given fields are updated.
#[derive(Debug, Deserialize, ToSchema)]
pub struct PatchTaskView {
    /// New title. Absent to leave it unchanged. Forbidden to an assigned agent who does not manage the project.
    #[schema(
        min_length = 1,
        max_length = 255,
        example = "Consult the residents and shopkeepers"
    )]
    pub name: Option<String>,
    /// New description, at most 5000 characters; `""` clears it. Absent to leave it unchanged.
    /// Forbidden to an assigned agent who does not manage the project.
    #[schema(max_length = 5000, example = "Public meeting with the residents")]
    pub description: Option<String>,
    /// New status category, the task taking the first status of it in its project unless it
    /// already is in that category. With `status_id`, the only fields an assigned agent who does not
    /// manage the project may change.
    pub status: Option<TaskStatus>,
    /// New status among the project's statuses (`GET …/statuses/`, MAIR-566); the task takes its
    /// category, whatever `status` says.
    #[serde(default)]
    #[schema(example = 31)]
    pub status_id: Option<u64>,
    /// New priority. Forbidden to an assigned agent who does not manage the project.
    pub priority: Option<TaskPriority>,
    /// New due date: an RFC 3339 date-time, or a date alone (`YYYY-MM-DD`) stored as midnight UTC
    /// of that day. Absent or `null` to leave it unchanged. Forbidden to an assigned agent who does
    /// not manage the project.
    #[serde(default, deserialize_with = "deserialize_optional_due_date")]
    #[schema(value_type = Option<String>, example = "2026-10-15")]
    pub due_date: Option<DateTime<Utc>>,
    /// New start date (MAIR-1087), same formats as `due_date`: absent to keep it, `null` to clear
    /// it. Must be at or before the due date, the one sent or the stored one (`400` otherwise).
    /// Forbidden to an assigned agent who does not manage the project.
    #[serde(default, deserialize_with = "deserialize_present_start_date")]
    #[schema(value_type = Option<String>, nullable, example = "2026-10-01")]
    pub start_date: Option<Option<DateTime<Utc>>>,
    /// Absent: assignment kept; `null`: assignment removed.
    #[serde(default, deserialize_with = "deserialize_present")]
    #[schema(value_type = Option<u64>, nullable, example = 42)]
    pub assigned_to: Option<Option<u64>>,
    /// New list of custom fields, replacing the whole stored list. Absent to leave it unchanged.
    /// Forbidden to an assigned agent who does not manage the project.
    pub fields: Option<Vec<DynamicTaskField>>,
}

impl PatchTaskView {
    /// `true` if the body only changes the status (`status` and/or `status_id`), the only change
    /// an assigned agent may make.
    pub fn only_changes_status(&self) -> bool {
        (self.status.is_some() || self.status_id.is_some())
            && self.name.is_none()
            && self.description.is_none()
            && self.priority.is_none()
            && self.due_date.is_none()
            && self.start_date.is_none()
            && self.assigned_to.is_none()
            && self.fields.is_none()
    }
}

impl Validate for PatchTaskView {
    fn validate(&self) -> Result<(), ValidationError> {
        check_optional(self.name.as_deref(), |name| {
            check_label("name", name.trim(), MAX_TITLE_LENGTH)
        })?;
        check_optional(self.description.as_deref(), |description| {
            check_description("description", description, MAX_DESCRIPTION_LENGTH)
        })?;
        check_dates_in_order(self.start_date.flatten(), self.due_date)?;
        for field in self.fields.iter().flatten() {
            check_label("fields.label", &field.label, MAX_TITLE_LENGTH)?;
            for option in &field.fields_options {
                check_json("fields.fields_options.option", &option.option)?;
            }
        }
        Ok(())
    }
}
