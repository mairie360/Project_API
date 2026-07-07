use std::fmt::Display;

use mairie360_api_lib::database::db_interface::DatabaseQueryView;

#[derive(Debug, sqlx::Type, PartialEq, Eq, Clone, Copy)]
#[sqlx(type_name = "project_status", rename_all = "lowercase")]
pub enum ProjectStatus {
    Active,
    Suspended,
    Archived,
    Completed,
    Error,
}

impl From<String> for ProjectStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "suspended" => ProjectStatus::Suspended,
            "archived" => ProjectStatus::Archived,
            "completed" => ProjectStatus::Completed,
            "error" => ProjectStatus::Error,
            _ => ProjectStatus::Active,
        }
    }
}

impl ToString for ProjectStatus {
    fn to_string(&self) -> String {
        match self {
            ProjectStatus::Active => "active".to_string(),
            ProjectStatus::Suspended => "suspended".to_string(),
            ProjectStatus::Archived => "archived".to_string(),
            ProjectStatus::Completed => "completed".to_string(),
            ProjectStatus::Error => "error".to_string(),
        }
    }
}

pub struct UpdateProjectStatusQueryView {
    project_id: u64,
    status: ProjectStatus,
}

impl UpdateProjectStatusQueryView {
    pub fn new(project_id: u64, status: ProjectStatus) -> Self {
        Self { project_id, status }
    }

    pub fn project_id(&self) -> u64 {
        self.project_id
    }

    pub fn status(&self) -> ProjectStatus {
        self.status
    }
}

impl Display for UpdateProjectStatusQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "UpdateProjectStatusQueryView: project_id={} status={}",
            self.project_id,
            self.status.to_string()
        )
    }
}

impl DatabaseQueryView for UpdateProjectStatusQueryView {
    fn get_request(&self) -> String {
        "UPDATE projects SET status = $1 WHERE id = $2".to_string()
    }
}
