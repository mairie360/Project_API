use utoipa::ToSchema;

use crate::endpoints::v1::projects::get::view::ProjectStatus;
use crate::endpoints::v1::projects::project_id::get::view::TaskPriority;
use crate::endpoints::validation::{
    check_description, check_label, check_optional, Validate, ValidationError,
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
    /// Group linked to the project: absent to keep it, `null` to remove it, an id to replace it.
    /// One of the caller's groups, any group for Admin and Maire (`GET /api/v1/project-groups/`).
    #[serde(default, deserialize_with = "present")]
    #[schema(value_type = Option<u64>, nullable = true, example = 3)]
    pub group_id: Option<Option<u64>>,
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
        check_optional(self.description.as_deref(), |description| {
            check_description("description", description, MAX_DESCRIPTION_LENGTH)
        })
    }
}
