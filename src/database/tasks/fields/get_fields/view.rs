use mairie360_api_lib::database::db_interface::DatabaseQueryView;
use sqlx::types::Json;
use std::{collections::HashMap, fmt::Display};

use crate::database::tasks::get_project_tasks::view::DynamicTaskField;

#[derive(Debug)]
pub struct GetTaskFieldsQueryView {
    id: u64,
}

impl GetTaskFieldsQueryView {
    pub fn new(id: u64) -> Self {
        Self { id }
    }

    pub fn id(&self) -> u64 {
        self.id
    }
}

impl DatabaseQueryView for GetTaskFieldsQueryView {
    fn get_request(&self) -> String {
        // La colonne 'custom_fields' est fusionnée avec le nouveau JSON ($2)
        "SELECT custom_fields FROM tasks WHERE id = $1".to_string()
    }
}

impl Display for GetTaskFieldsQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_request())
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct CustomeFieldsQueryResultView {
    custom_fields: Json<HashMap<String, DynamicTaskField>>,
}

impl CustomeFieldsQueryResultView {
    pub fn custom_fields(&self) -> &Json<HashMap<String, DynamicTaskField>> {
        &self.custom_fields
    }
}
