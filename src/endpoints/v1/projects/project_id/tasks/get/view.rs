use chrono::{DateTime, Utc};
use utoipa::ToSchema;

use crate::endpoints::v1::projects::project_id::get::view::{TaskPriority, TaskStatus};

#[derive(Debug, serde::Deserialize, serde::Serialize, ToSchema)]
pub enum TaskFieldType {
    #[schema(value_type = String, format = DateTime)]
    Date(DateTime<Utc>),
    CheckBox(Vec<String>),
    Select(String),
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct TaskField {
    pub field_type: TaskFieldType,
    pub name: String,
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct TaskView {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    #[schema(value_type = String, format = DateTime)]
    pub due_date: Option<DateTime<Utc>>,
    pub assigned_to: Option<u64>,
    pub fields: Vec<TaskField>,
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetTasksResultView {
    pub tasks: Vec<TaskView>,
}
