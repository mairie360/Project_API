use chrono::{DateTime, Utc};
use mairie360_api_lib::database::db_interface::id_from_sql;
use utoipa::ToSchema;

use crate::{
    database::tasks::get_project_tasks::view::{DynamicTaskField, Task},
    endpoints::v1::projects::project_id::get::view::{TaskPriority, TaskStatus},
};

/// Task of a project.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct TaskView {
    /// Id of the task.
    #[schema(example = 77)]
    pub id: u64,
    /// Title of the task.
    #[schema(example = "Consult the residents")]
    pub title: String,
    /// Description of the task; empty string if it has none — never `null`.
    #[schema(example = "Public meeting to organise before 15 October")]
    pub description: String,
    /// Current status. `Error` flags a stored value the API cannot interpret.
    pub status: TaskStatus,
    /// Id of the task's status among its project's statuses (`GET /api/v1/projects/{project_id}/statuses/`,
    /// MAIR-566); `status` is its category.
    #[schema(example = 31)]
    pub status_id: Option<u64>,
    /// Name of the task's status in its project (e.g. `À valider`), `null` only for a task read
    /// while its status is being deleted.
    #[schema(example = "En cours")]
    pub status_name: Option<String>,
    /// Priority. `Error` flags a stored value the API cannot interpret.
    pub priority: TaskPriority,
    /// Due date, or `null` if the task has none.
    #[schema(value_type = Option<String>, format = DateTime, example = "2026-10-15T00:00:00Z")]
    pub due_date: Option<DateTime<Utc>>,
    /// Start date (MAIR-1087), at or before `due_date`, or `null` if the task has none.
    #[schema(value_type = Option<String>, format = DateTime, example = "2026-10-01T00:00:00Z")]
    pub start_date: Option<DateTime<Utc>>,
    /// When the task was archived (MAIR-502: a task is archived as soon as it is `Completed`, and
    /// back when it is reopened), or `null` while it is active.
    #[schema(value_type = Option<String>, format = DateTime, example = json!(null))]
    pub archived_at: Option<DateTime<Utc>>,
    /// Core API id of the assigned agent, or `null` if the task is assigned to nobody.
    #[schema(example = 42)]
    pub assigned_to: Option<u64>,
    /// Custom fields defined on the project. Empty if there are none.
    pub fields: Vec<DynamicTaskField>,
}

impl From<Task> for TaskView {
    fn from(task: Task) -> Self {
        TaskView {
            id: id_from_sql(task.id()),
            title: task.title().to_string(),
            description: task.description().to_string(),
            status: task.status().to_string().into(),
            status_id: task.status_id().map(id_from_sql),
            status_name: task.status_name().map(str::to_string),
            priority: task.priority().to_string().into(),
            due_date: task.due_date(),
            start_date: task.start_date(),
            archived_at: task.archived_at(),
            assigned_to: task.assigned_to().map(id_from_sql),
            fields: task.fields(),
        }
    }
}

/// One page of the tasks of a project.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetTasksResultView {
    /// Tasks of the page, oldest first. Empty if the project has none or the page is past the end.
    pub tasks: Vec<TaskView>,
    /// Number of tasks of the project, whatever the page.
    #[schema(example = 1)]
    pub total: u64,
}
