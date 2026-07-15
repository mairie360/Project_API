use std::fmt::Display;

use mairie360_api_lib::database::db_interface::DatabaseQueryView;
use sqlx::types::Json;

use crate::database::tasks::get_project_tasks::view::DynamicTaskField;

#[derive(Debug)]
pub struct AddFieldToTaskQueryView {
    task_id: u64,
    custom_fields: Json<std::collections::HashMap<String, DynamicTaskField>>,
}

impl AddFieldToTaskQueryView {
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

impl DatabaseQueryView for AddFieldToTaskQueryView {
    fn get_request(&self) -> String {
        // La colonne 'custom_fields' est fusionnée avec le nouveau JSON ($2)
        "UPDATE tasks
         SET custom_fields = custom_fields || $2::jsonb
         WHERE id = $1"
            .to_string()
    }
}

impl Display for AddFieldToTaskQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AddFieldToTaskQueryView: task_id={}, custom_fields={:?}",
            self.task_id, self.custom_fields
        )
    }
}
