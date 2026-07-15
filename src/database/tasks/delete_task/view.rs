use std::fmt::Display;

use mairie360_api_lib::database::db_interface::DatabaseQueryView;

pub struct DeleteTaskQueryView {
    task_id: u64,
}

impl DeleteTaskQueryView {
    pub fn new(task_id: u64) -> Self {
        Self { task_id }
    }

    pub fn task_id(&self) -> u64 {
        self.task_id
    }
}

impl Display for DeleteTaskQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DeleteTaskQueryView: task_id={}", self.task_id)
    }
}

impl DatabaseQueryView for DeleteTaskQueryView {
    fn get_request(&self) -> String {
        "DELETE FROM tasks WHERE id = $1".to_string()
    }
}
