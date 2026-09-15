use utoipa::ToSchema;

use crate::endpoints::v1::projects::get::view::ProjectStatus;

#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct UpdateProjectView {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<ProjectStatus>,
}
