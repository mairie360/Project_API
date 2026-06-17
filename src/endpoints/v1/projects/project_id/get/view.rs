use chrono::{DateTime, Utc};
use utoipa::ToSchema;

#[derive(Debug, serde::Deserialize, serde::Serialize, ToSchema)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Completed,
    Error,
}

impl From<String> for TaskStatus {
    fn from(value: String) -> Self {
        match value.as_str() {
            "todo" => TaskStatus::Todo,
            "in_progress" => TaskStatus::InProgress,
            "completed" => TaskStatus::Completed,
            _ => TaskStatus::Error,
        }
    }
}

impl ToString for TaskStatus {
    fn to_string(&self) -> String {
        match self {
            TaskStatus::Todo => "todo".to_string(),
            TaskStatus::InProgress => "in_progress".to_string(),
            TaskStatus::Completed => "completed".to_string(),
            TaskStatus::Error => "error".to_string(),
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize, ToSchema)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Urgent,
    Error,
}

impl From<String> for TaskPriority {
    fn from(value: String) -> Self {
        match value.as_str() {
            "low" => TaskPriority::Low,
            "medium" => TaskPriority::Medium,
            "high" => TaskPriority::High,
            "urgent" => TaskPriority::Urgent,
            _ => TaskPriority::Error,
        }
    }
}

impl ToString for TaskPriority {
    fn to_string(&self) -> String {
        match self {
            TaskPriority::Low => "low".to_string(),
            TaskPriority::Medium => "medium".to_string(),
            TaskPriority::High => "high".to_string(),
            TaskPriority::Urgent => "urgent".to_string(),
            TaskPriority::Error => "error".to_string(),
        }
    }
}

#[derive(Debug, serde::Serialize, ToSchema)]
struct TaskView {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    #[schema(value_type = String, format = DateTime)]
    pub due_date: Option<DateTime<Utc>>,
    pub assigned_to: Option<u64>,
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetProjectResultView {
    pub name: String,
    pub description: String,
    pub tasks: Vec<TaskView>,
}
