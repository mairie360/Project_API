use utoipa::ToSchema;

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

impl ToString for ProjectStatus {
    fn to_string(&self) -> String {
        match self {
            ProjectStatus::Active => "active".to_string(),
            ProjectStatus::Suspended => "suspended".to_string(),
            ProjectStatus::Completed => "completed".to_string(),
            ProjectStatus::Error => "error".to_string(),
        }
    }
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct ProjetView {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub status: ProjectStatus,
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetProjectsResultView {
    pub projects: Vec<ProjetView>,
}
