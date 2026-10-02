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
    pub priority: Option<TaskPriority>,
    pub due_date: Option<chrono::DateTime<chrono::Utc>>,
    pub assigned_to: Option<Option<u64>>,
    pub fields: Option<&'a [DynamicTaskField]>,
}

/// Updates a task of a project, signed by `updated_by` (the history trigger records the changes under
/// that author). Returns `true` if the task exists in this project and the new assignee, if any, owns
/// or is a member of the project; nothing is written otherwise.
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
                    priority = COALESCE(NULLIF($5, '')::task_priority, priority), \
                    due_date = COALESCE(NULLIF($6, '')::timestamptz AT TIME ZONE 'UTC', due_date), \
                    assigned_to = CASE WHEN $7 THEN $8 ELSE assigned_to END, \
                    description = CASE WHEN $9 THEN $10 ELSE description END, \
                    custom_fields = CASE WHEN $11 = '' THEN custom_fields \
                        ELSE jsonb_set(COALESCE(custom_fields, '{}'::jsonb), '{fields}', $11::jsonb, true) END, \
                    updated_by = $12, \
                    updated_at = CURRENT_TIMESTAMP \
                WHERE id = $1 AND project_id = $2 \
                  AND (NOT $7 OR $8::int IS NULL OR ",
            crate::assignable_to_project_sql!("$2", "$8"),
            ") RETURNING id \
             ) \
             SELECT EXISTS (SELECT 1 FROM updated)"
        )
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
