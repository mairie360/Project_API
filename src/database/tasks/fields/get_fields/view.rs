use std::collections::HashMap;

use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

use crate::database::tasks::get_project_tasks::view::DynamicTaskField;

/// Type de résultat renvoyé par [`GetTaskFieldsQueryView`] : la map des champs
/// dynamiques stockés dans la colonne `custom_fields` de la tâche.
pub type TaskCustomFields = HashMap<String, DynamicTaskField>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetTaskFieldsQueryView {
    params: Vec<QueryParam>,
}

impl GetTaskFieldsQueryView {
    pub fn new(id: u64) -> Self {
        Self {
            params: vec![QueryParam::I32(id as i32)],
        }
    }

    pub fn id(&self) -> u64 {
        self.params[0].as_i32() as u64
    }
}

impl ApiRequestDto for GetTaskFieldsQueryView {
    fn query_sql(&self) -> &'static str {
        "SELECT COALESCE(custom_fields, '{}'::jsonb) FROM tasks WHERE id = $1"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
