use std::collections::HashMap;

use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetProjectTasksQueryView {
    params: Vec<QueryParam>,
}

impl GetProjectTasksQueryView {
    pub fn new(project_id: u64) -> Self {
        Self {
            params: vec![QueryParam::I32(project_id as i32)],
        }
    }

    pub fn project_id(&self) -> u64 {
        self.params[0].as_i32() as u64
    }
}

impl ApiRequestDto for GetProjectTasksQueryView {
    fn query_sql(&self) -> &'static str {
        "SELECT to_jsonb(t) FROM ( \
            SELECT id, title, status, priority, created_at, assigned_to, \
                   COALESCE(custom_fields, '{}'::jsonb) AS custom_fields \
            FROM tasks WHERE project_id = $1 \
         ) t"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy, ToSchema)]
#[serde(rename_all = "lowercase")] // Magique : transforme "Date" en "date" dans le JSON
pub enum FieldType {
    Date,
    Checkbox,
    Select,
    #[serde(other)] // Gère les types inconnus proprement (remplace ton "Error")
    Unknown,
}

// 2. Les options du champ
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, ToSchema)]
pub struct FieldOption {
    pub option: serde_json::Value,
    pub is_selected: bool,
}

// 3. Le champ dynamique
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, ToSchema)]
pub struct DynamicTaskField {
    pub label: String,
    pub task_type: FieldType,
    pub fields_options: Vec<FieldOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    id: i32,
    title: String,
    status: String,
    priority: String,
    #[serde(default)]
    created_at: Option<String>,
    #[serde(default)]
    assigned_to: Option<i32>,
    #[serde(default)]
    custom_fields: HashMap<String, DynamicTaskField>,
}

impl Task {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn priority(&self) -> &str {
        &self.priority
    }

    pub fn created_at(&self) -> Option<&str> {
        self.created_at.as_deref()
    }

    pub fn assigned_to(&self) -> Option<i32> {
        self.assigned_to
    }

    pub fn custom_fields(&self) -> &HashMap<String, DynamicTaskField> {
        &self.custom_fields
    }
}
