use utoipa::ToSchema;

use crate::database::project::get_projects::view::ProjectView;

#[derive(Debug, serde::Serialize, ToSchema)]
pub enum ProjectStatus {
    Active,
    Suspended,
    Completed,
    Error,
}

impl From<String> for ProjectStatus {
    fn from(value: String) -> Self {
        match value.as_str() {
            "active" => ProjectStatus::Active,
            "suspended" => ProjectStatus::Suspended,
            "completed" => ProjectStatus::Completed,
            _ => ProjectStatus::Error,
        }
    }
}

impl std::fmt::Display for ProjectStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ProjectStatus::Active => "active",
            ProjectStatus::Suspended => "suspended",
            ProjectStatus::Completed => "completed",
            ProjectStatus::Error => "error",
        };
        f.write_str(s)
    }
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct ProjetView {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub status: ProjectStatus,
}

impl From<ProjectView> for ProjetView {
    fn from(value: ProjectView) -> Self {
        Self {
            id: value.id() as u64,
            name: value.title().to_string(),
            description: value.description().unwrap_or_default().to_string(),
            status: value.status().to_string().into(),
        }
    }
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetProjectsResultView {
    pub projects: Vec<ProjetView>,
}
