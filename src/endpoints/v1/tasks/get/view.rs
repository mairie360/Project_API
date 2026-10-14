use chrono::{DateTime, Utc};
use mairie360_api_lib::database::db_interface::id_from_sql;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use crate::database::paged::PagedRows;
use crate::database::tasks::get_project_tasks::view::DynamicTaskField;
use crate::database::tasks::get_visible_tasks::view::{TaskFilters, VisibleTaskRow};
use crate::endpoints::pagination::{Page, PageParams};
use crate::endpoints::v1::projects::project_id::get::view::{TaskPriority, TaskStatus};
use crate::endpoints::validation::{check_list, check_optional, Validate, ValidationError};

/// Values of `status`, and the stored status group each one keeps.
const STATUS_FILTERS: [(&str, &str); 4] = [
    ("Todo", "todo"),
    ("InProgress", "in_progress"),
    ("Completed", "completed"),
    ("Error", "other"),
];

/// Page and filters of `GET /api/v1/tasks/` (MAIR-524). Every filter is optional and they combine
/// (AND); `total` counts the tasks that match them all.
#[derive(Debug, Default, Clone, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct GetTasksQuery {
    /// Maximum number of tasks returned, from 1 to 500. Defaults to 100. `0` is raised to 1 and a
    /// value above 500 is lowered to 500.
    #[param(minimum = 1, maximum = 500, default = 100, example = 20)]
    limit: Option<u32>,
    /// Number of matching tasks skipped before the first one returned. Defaults to 0. Past the end,
    /// the page is empty but `total` is still the full count.
    #[param(minimum = 0, default = 0, example = 0)]
    offset: Option<u32>,
    /// Keeps the tasks assigned to this Core API user id (the caller's own id for "my pending
    /// tasks"). Any user id is accepted: only the tasks of the projects the caller sees are listed
    /// either way, and an unknown id gives an empty list.
    #[param(minimum = 1, example = 42)]
    assigned_to: Option<u64>,
    /// Keeps the tasks with one of these statuses: a comma-separated list of `Todo`, `InProgress`,
    /// `Completed` and `Error` (a stored status the API does not interpret, e.g. `review`).
    /// Without it, every **active** task is listed (MAIR-502: a `Completed` task is archived and
    /// left out). Archived tasks are only listed when `Completed` is in the list.
    #[param(example = "Todo,InProgress")]
    status: Option<String>,
    /// Keeps the tasks due at or before this instant (RFC 3339). A task without due date never
    /// matches.
    #[param(value_type = Option<String>, format = DateTime, example = "2026-10-31T23:59:59Z")]
    due_before: Option<DateTime<Utc>>,
}

impl GetTasksQuery {
    pub fn page(&self) -> Page {
        PageParams::new(self.limit, self.offset).page()
    }

    /// The filters, as database codes (call after `validate`).
    pub fn filters(&self) -> TaskFilters {
        TaskFilters {
            assigned_to: self.assigned_to,
            statuses: self.status.as_deref().map_or_else(Vec::new, |status| {
                status
                    .split(',')
                    .filter_map(|item| {
                        STATUS_FILTERS
                            .iter()
                            .find(|(name, _)| *name == item)
                            .map(|(_, code)| *code)
                    })
                    .collect()
            }),
            due_before: self.due_before,
        }
    }
}

impl Validate for GetTasksQuery {
    fn validate(&self) -> Result<(), ValidationError> {
        let names: Vec<&str> = STATUS_FILTERS.iter().map(|(name, _)| *name).collect();
        check_optional(self.status.as_deref(), |status| {
            check_list("status", status, &names)
        })
    }
}

/// A task of the cross-project list (MAIR-524): the fields of a project's task
/// (`GET /api/v1/projects/{project_id}/tasks/{task_id}/`) and the project it belongs to.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct TaskListItemView {
    /// Id of the task, to use in `/api/v1/projects/{project_id}/tasks/{task_id}/` with `project_id`.
    #[schema(example = 87)]
    pub id: u64,
    /// Title of the task.
    #[schema(example = "Consulter les riverains")]
    pub title: String,
    /// Description of the task; empty string if it has none — never `null`.
    #[schema(example = "Réunion publique à organiser avant le 15 octobre")]
    pub description: String,
    /// Current status. `Error` flags a stored value the API does not interpret.
    pub status: TaskStatus,
    /// Priority. `Error` flags a stored value the API does not interpret.
    pub priority: TaskPriority,
    /// Due date, or `null` if the task has none (listed last).
    #[schema(value_type = Option<String>, format = DateTime, example = "2026-10-15T00:00:00Z")]
    pub due_date: Option<DateTime<Utc>>,
    /// When the task was archived (it became `Completed`), or `null` while it is active. Only
    /// archived tasks have one, and they are only listed when `status` includes `Completed`.
    #[schema(value_type = Option<String>, format = DateTime, example = json!(null))]
    pub archived_at: Option<DateTime<Utc>>,
    /// Core API id of the assigned agent, or `null` if the task is assigned to nobody.
    #[schema(example = 42)]
    pub assigned_to: Option<u64>,
    /// Custom fields of the task. Empty if it has none.
    pub fields: Vec<DynamicTaskField>,
    /// Id of the task's project, to use in `/api/v1/projects/{project_id}/`.
    #[schema(example = 12)]
    pub project_id: u64,
    /// Name of the task's project.
    #[schema(example = "Réfection de la place du marché")]
    pub project_name: String,
}

impl From<VisibleTaskRow> for TaskListItemView {
    fn from(row: VisibleTaskRow) -> Self {
        let task = row.task;
        Self {
            id: id_from_sql(task.id()),
            title: task.title().to_string(),
            description: task.description().to_string(),
            status: task.status().to_string().into(),
            priority: task.priority().to_string().into(),
            due_date: task.due_date(),
            archived_at: task.archived_at(),
            assigned_to: task.assigned_to().map(id_from_sql),
            fields: task.fields(),
            project_id: id_from_sql(row.project_id),
            project_name: row.project_name,
        }
    }
}

/// One page of the tasks visible to the caller across every project.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetTasksListResultView {
    /// Tasks of the page, by due date (soonest first, tasks without due date last), then id.
    /// Empty if no visible task matches the filters or the page is past the end.
    pub tasks: Vec<TaskListItemView>,
    /// Number of visible tasks matching the filters, whatever the page.
    #[schema(example = 2)]
    pub total: u64,
}

impl From<PagedRows<VisibleTaskRow>> for GetTasksListResultView {
    fn from(page: PagedRows<VisibleTaskRow>) -> Self {
        Self {
            tasks: page.items.into_iter().map(Into::into).collect(),
            total: u64::try_from(page.total).unwrap_or(0),
        }
    }
}
