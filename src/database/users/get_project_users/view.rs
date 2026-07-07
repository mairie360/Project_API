use std::fmt::Display;

use mairie360_api_lib::database::db_interface::DatabaseQueryView;

pub struct GetProjectUsersQueryView {
    project_id: u64,
}

impl GetProjectUsersQueryView {
    pub fn new(project_id: u64) -> Self {
        Self { project_id }
    }

    pub fn project_id(&self) -> u64 {
        self.project_id
    }
}

impl Display for GetProjectUsersQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "GetProjectUsersQueryView: project_id={}",
            self.project_id
        )
    }
}

impl DatabaseQueryView for GetProjectUsersQueryView {
    fn get_request(&self) -> String {
        "SELECT user_id FROM project_members WHERE project_id = $1".to_string()
    }
}
