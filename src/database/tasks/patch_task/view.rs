use mairie360_api_lib::database::db_interface::{
    id_from_sql, id_to_sql, ApiRequestDto, QueryParam,
};

use crate::database::tasks::create_task::view::{TaskPriority, TaskStatus};
use crate::database::tasks::get_project_tasks::view::DynamicTaskField;

/// Changes requested on a task. A `None` field is kept; for `assigned_to`, `Some(None)` removes the
/// assignee.
#[derive(Debug, Clone, Default)]
pub struct TaskChanges<'a> {
    pub title: Option<&'a str>,
    pub description: Option<&'a str>,
    pub status: Option<TaskStatus>,
    /// A status of the task's project (MAIR-566); the database sets the category from it.
    pub status_id: Option<u64>,
    pub priority: Option<TaskPriority>,
    pub due_date: Option<chrono::DateTime<chrono::Utc>>,
    pub assigned_to: Option<Option<u64>>,
    pub fields: Option<&'a [DynamicTaskField]>,
}

/// [`PatchTaskQueryView`]: the task was updated.
pub const TASK_UPDATED: i32 = 1;
/// The task is not in this project, or the new assignee neither owns nor is a member of it.
pub const TASK_NOT_UPDATED: i32 = 0;
/// `status_id` is not a status of the task's project (MAIR-566).
pub const TASK_STATUS_NOT_IN_PROJECT: i32 = -1;

/// Updates a task of a project, signed by `updated_by` (the history trigger records the changes under
/// that author). Returns [`TASK_UPDATED`], or [`TASK_STATUS_NOT_IN_PROJECT`] / [`TASK_NOT_UPDATED`]
/// when nothing is written.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PatchTaskQueryView {
    params: Vec<QueryParam>,
}

impl PatchTaskQueryView {
    pub fn new(project_id: u64, task_id: u64, updated_by: u64, changes: TaskChanges<'_>) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(task_id)),
                QueryParam::I32(id_to_sql(project_id)),
                QueryParam::Text(changes.title.unwrap_or_default().to_string()),
                QueryParam::Text(changes.status.map(|s| s.to_string()).unwrap_or_default()),
                QueryParam::Text(changes.priority.map(|p| p.to_string()).unwrap_or_default()),
                QueryParam::Text(changes.due_date.map(|d| d.to_rfc3339()).unwrap_or_default()),
                QueryParam::Bool(changes.assigned_to.is_some()),
                QueryParam::OptionI32(changes.assigned_to.flatten().map(id_to_sql)),
                QueryParam::Bool(changes.description.is_some()),
                QueryParam::Text(changes.description.unwrap_or_default().to_string()),
                QueryParam::Text(
                    changes
                        .fields
                        .map(|fields| serde_json::json!(fields).to_string())
                        .unwrap_or_default(),
                ),
                QueryParam::I32(id_to_sql(updated_by)),
                QueryParam::OptionI32(changes.status_id.map(id_to_sql)),
            ],
        }
    }

    pub fn task_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }

    pub fn project_id(&self) -> u64 {
        id_from_sql(self.params[1].as_i32())
    }
}

impl ApiRequestDto for PatchTaskQueryView {
    fn query_sql(&self) -> &'static str {
        // due_date is a TIMESTAMP column stored in UTC. An empty description is a valid value, hence the
        // explicit `$9` flag instead of NULLIF.
        concat!(
            "WITH updated AS ( \
                UPDATE tasks SET \
                    title = COALESCE(NULLIF($3, ''), title), \
                    status = COALESCE(NULLIF($4, '')::task_status, status), \
                    status_id = COALESCE($13::int, status_id), \
                    priority = COALESCE(NULLIF($5, '')::task_priority, priority), \
                    due_date = COALESCE(NULLIF($6, '')::timestamptz AT TIME ZONE 'UTC', due_date), \
                    assigned_to = CASE WHEN $7 THEN $8 ELSE assigned_to END, \
                    description = CASE WHEN $9 THEN $10 ELSE description END, \
                    custom_fields = CASE WHEN $11 = '' THEN custom_fields \
                        ELSE jsonb_set(COALESCE(custom_fields, '{}'::jsonb), '{fields}', $11::jsonb, true) END, \
                    updated_by = $12, \
                    updated_at = CURRENT_TIMESTAMP \
                WHERE id = $1 AND project_id = $2 AND NOT ",
            crate::status_outside_project_sql!("$2", "$13"),
            " AND (NOT $7 OR $8::int IS NULL OR ",
            crate::assignable_to_project_sql!("$2", "$8"),
            ") RETURNING id \
             ) \
             SELECT CASE WHEN EXISTS (SELECT 1 FROM updated) THEN 1 \
                         WHEN ",
            crate::status_outside_project_sql!("$2", "$13"),
            " THEN -1 ELSE 0 END"
        )
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }

    fn invalidated_read_caches(&self) -> &'static [&'static str] {
        crate::database::project::PROJECTS_READ_CACHES
    }
}
