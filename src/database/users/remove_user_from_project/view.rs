use std::fmt::Display;

use mairie360_api_lib::database::db_interface::DatabaseQueryView;

pub struct RemoveUserFromProjectQueryView {
    project_id: u64,
    user_id: u64,
}

impl RemoveUserFromProjectQueryView {
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

impl Display for RemoveUserFromProjectQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RemoveUserFromProjectQueryView: project_id={} user_id={}",
            self.project_id, self.user_id
        )
    }
}

impl DatabaseQueryView for RemoveUserFromProjectQueryView {
    fn get_request(&self) -> String {
        "DELETE FROM project_members WHERE project_id = $1 AND user_id = $2".to_string()
    }
}
