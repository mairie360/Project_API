use std::fmt::Display;

use mairie360_api_lib::database::db_interface::DatabaseQueryView;

pub struct AddUserToProjectQueryView {
    project_id: u64,
    user_id: u64,
}

impl AddUserToProjectQueryView {
    pub fn new(project_id: u64, user_id: u64) -> Self {
        Self {
            project_id,
            user_id,
        }
    }

    pub fn project_id(&self) -> u64 {
        self.project_id
    }

    pub fn user_id(&self) -> u64 {
        self.user_id
    }
}

impl Display for AddUserToProjectQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AddUserToProjectQueryView: project_id={} user_id={}",
            self.project_id, self.user_id
        )
    }
}

impl DatabaseQueryView for AddUserToProjectQueryView {
    fn get_request(&self) -> String {
        "INSERT INTO project_members (project_id, user_id) VALUES ($1, $2)".to_string()
    }
}
