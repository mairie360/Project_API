use std::fmt::Display;

use mairie360_api_lib::database::db_interface::DatabaseQueryView;

pub struct DeleteProjectQueryView {
    project_id: u64,
}

impl DeleteProjectQueryView {
    pub fn new(project_id: u64) -> Self {
        Self { project_id }
    }

    pub fn project_id(&self) -> u64 {
        self.project_id
    }
}

impl Display for DeleteProjectQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DeleteProjectQueryView: project_id={}", self.project_id)
    }
}

impl DatabaseQueryView for DeleteProjectQueryView {
    fn get_request(&self) -> String {
        "DELETE FROM projects WHERE id = $1".to_string()
    }
}
