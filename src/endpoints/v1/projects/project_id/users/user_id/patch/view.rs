use utoipa::ToSchema;

use crate::endpoints::v1::projects::project_id::users::get::view::ProjectMemberRole;

/// New role of a member of the project (MAIR-1088).
#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct UpdateMemberRoleView {
    /// `manager`, `contributor` or `reader`. Required.
    pub role: ProjectMemberRole,
}
