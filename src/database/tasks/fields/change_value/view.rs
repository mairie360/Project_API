use std::fmt::Display;

use mairie360_api_lib::database::db_interface::DatabaseQueryView;
use sqlx::types::Json;

use crate::database::tasks::get_project_tasks::view::DynamicTaskField;

#[derive(Debug)]
pub struct ChangeFieldValueQueryView {
    task_id: u64,
    custom_fields: Json<std::collections::HashMap<String, DynamicTaskField>>,
}

impl ChangeFieldValueQueryView {
    pub fn new(
        task_id: u64,
        custom_fields: Json<std::collections::HashMap<String, DynamicTaskField>>,
    ) -> Self {
        Self {
            task_id,
            custom_fields,
        }
    }

    pub fn task_id(&self) -> u64 {
        self.task_id
    }

    pub fn custom_fields(&self) -> &Json<std::collections::HashMap<String, DynamicTaskField>> {
        &self.custom_fields
    }
}

impl DatabaseQueryView for ChangeFieldValueQueryView {
    fn get_request(&self) -> String {
        "UPDATE tasks
         SET custom_fields = $2::jsonb
         WHERE id = $1"
            .to_string()
    }
}

impl Display for ChangeFieldValueQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ChangeFieldValueQueryView: task_id={}, custom_fields={:?}",
            self.task_id, self.custom_fields
        )
    }
}
