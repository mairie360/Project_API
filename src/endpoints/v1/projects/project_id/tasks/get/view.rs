use chrono::{DateTime, Utc};
use utoipa::ToSchema;

use crate::{
    database::tasks::get_project_tasks::view::{DynamicTaskField, Task},
    endpoints::v1::projects::project_id::get::view::{TaskPriority, TaskStatus},
};

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct TaskView {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    #[schema(value_type = Option<String>, format = DateTime)]
    pub due_date: Option<DateTime<Utc>>,
    pub assigned_to: Option<u64>,
    pub fields: Vec<DynamicTaskField>,
}

impl From<Task> for TaskView {
    fn from(task: Task) -> Self {
        TaskView {
            id: task.id() as u64,
            title: task.title().to_string(),
            description: "".to_string(),
            status: task.status().to_string().into(),
            priority: task.priority().to_string().into(),
            due_date: task.due_date(),
            assigned_to: task.assigned_to().map(|id| id as u64),
            fields: task.fields(),
        }
    }
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetTasksResultView {
    pub tasks: Vec<TaskView>,
}
