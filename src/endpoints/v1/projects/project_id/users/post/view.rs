use utoipa::ToSchema;

use crate::endpoints::v1::projects::project_id::users::get::view::ProjectMemberRole;

/// User to add to the project of the path.
#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct AddUserToProjectView {
    /// Core API id of the user to add, as returned by
    /// `GET /api/v1/user/` of Core API.
    #[schema(example = 42)]
    pub user_id: u64,
    /// Role of the new member (MAIR-1088). Absent or `null`: `contributor`.
    #[serde(default)]
    pub role: Option<ProjectMemberRole>,
}
