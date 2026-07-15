use std::fmt::Display;

use mairie360_api_lib::database::db_interface::DatabaseQueryView;
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use sqlx::FromRow;
use utoipa::ToSchema;

use crate::database::tasks::create_task::view::{TaskPriority, TaskStatus};

pub struct GetProjectTasksQueryView {
    project_id: u64,
}

impl GetProjectTasksQueryView {
    pub fn new(project_id: u64) -> Self {
        Self { project_id }
    }

    pub fn project_id(&self) -> u64 {
        self.project_id
    }
}

impl Display for GetProjectTasksQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "GetProjectTasksQueryView: project_id={}",
            self.project_id
        )
    }
}

impl DatabaseQueryView for GetProjectTasksQueryView {
    fn get_request(&self) -> String {
        "SELECT id, title, status, priority, created_at, assigned_to, custom_fields FROM tasks WHERE project_id = $1".to_string()
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

#[derive(Debug, FromRow)]
pub struct Task {
    id: i32,
    title: String,
    status: TaskStatus,
    priority: TaskPriority,
    created_at: Option<chrono::NaiveDateTime>,
    assigned_to: Option<i32>,
    custom_fields: Json<std::collections::HashMap<String, DynamicTaskField>>,
}

impl Task {
    pub fn new(
        id: i32,
        title: String,
        status: TaskStatus,
        priority: TaskPriority,
        created_at: Option<chrono::NaiveDateTime>,
        assigned_to: Option<i32>,
        custom_fields: Json<std::collections::HashMap<String, DynamicTaskField>>,
    ) -> Self {
        Self {
            id,
            title,
            status,
            priority,
            created_at,
            assigned_to,
            custom_fields,
        }
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn status(&self) -> &TaskStatus {
        &self.status
    }

    pub fn priority(&self) -> &TaskPriority {
        &self.priority
    }

    pub fn created_at(&self) -> &Option<chrono::NaiveDateTime> {
        &self.created_at
    }

    pub fn assigned_to(&self) -> &Option<i32> {
        &self.assigned_to
    }

    pub fn custom_fields(&self) -> &Json<std::collections::HashMap<String, DynamicTaskField>> {
        &self.custom_fields
    }
}
