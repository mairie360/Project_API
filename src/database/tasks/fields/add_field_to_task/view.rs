use std::collections::HashMap;

use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

use crate::database::tasks::get_project_tasks::view::DynamicTaskField;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AddFieldToTaskQueryView {
    params: Vec<QueryParam>,
}

impl AddFieldToTaskQueryView {
    pub fn new(task_id: u64, custom_fields: HashMap<String, DynamicTaskField>) -> Self {
        Self {
            params: vec![
                QueryParam::I32(task_id as i32),
                QueryParam::Text(
                    serde_json::to_string(&custom_fields).unwrap_or_else(|_| "{}".to_string()),
                ),
            ],
        }
    }

    pub fn task_id(&self) -> u64 {
        self.params[0].as_i32() as u64
    }
}

impl ApiRequestDto for AddFieldToTaskQueryView {
    fn query_sql(&self) -> &'static str {
        // La colonne 'custom_fields' est fusionnée avec le nouveau JSON ($2)
        "UPDATE tasks \
         SET custom_fields = COALESCE(custom_fields, '{}'::jsonb) || $2::jsonb \
         WHERE id = $1"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
