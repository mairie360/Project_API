use chrono::{DateTime, Utc};
use mairie360_api_lib::database::db_interface::{
    id_from_sql, id_to_sql, ApiRequestDto, QueryParam,
};
use serde::{Deserialize, Serialize};

use crate::database::tasks::get_project_tasks::view::Task;

/// Filters of the cross-project task list (MAIR-524). Every filter is optional; the values are the
/// database codes, already checked by the endpoint.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TaskFilters {
    /// Keeps the tasks assigned to this user.
    pub assigned_to: Option<u64>,
    /// Status groups kept: `todo`, `in_progress`, `completed`, `other` (any other stored value).
    /// Empty: every status of the active tasks.
    pub statuses: Vec<&'static str>,
    /// Keeps the tasks due at or before this instant (a task without due date never matches).
    pub due_before: Option<DateTime<Utc>>,
}

impl TaskFilters {
    /// Archived tasks are only listed when `completed` is asked for: a task is archived exactly
    /// when it is completed (MAIR-502), so every other list stays on the active tasks and their
    /// partial indexes.
    pub fn includes_archived(&self) -> bool {
        self.statuses.contains(&"completed")
    }
}

/// `SELECT` of the cross-project task list, its `WHERE` prefix (assignee, archived) given as a
/// literal so each variant is its own statement (no `$n IS NULL OR …` defeating the indexes).
///
/// The visible projects are collected once (`project_visible_to_user_sql!`, user `$1`) in a
/// materialized CTE, then joined to the tasks: with an assignee, the range of
/// `idx_tasks_assignee_due` (`assigned_to, due_date, id WHERE archived_at IS NULL`, Database
/// v3.1.0; `idx_tasks_assigned_to` before it); without, `idx_tasks_active_due` or the tasks of the
/// visible projects through `idx_tasks_project_id`. `total` and the page are planned separately
/// (`NOT MATERIALIZED`), so the page stops at `LIMIT`.
macro_rules! visible_tasks_sql {
    ($where:literal) => {
        concat!(
            "WITH visible AS MATERIALIZED ( \
                SELECT p.id, p.title FROM projects p WHERE ",
            crate::project_visible_to_user_sql!(),
            " ), matched AS NOT MATERIALIZED ( \
                SELECT t.id, t.title, COALESCE(t.description, '') AS description, t.status, \
                       t.priority, t.created_at, t.assigned_to, t.due_date, t.archived_at, \
                       COALESCE(t.custom_fields, '{}'::jsonb) AS custom_fields, \
                       v.id AS project_id, v.title AS project_name \
                FROM tasks t JOIN visible v ON v.id = t.project_id \
                WHERE ",
            $where,
            " ($2 = '' OR (CASE WHEN t.status::text IN ('todo', 'in_progress', 'completed') \
                               THEN t.status::text ELSE 'other' END) = ANY (string_to_array($2, ','))) \
                  AND (NOT $4 OR t.due_date <= $3::timestamptz AT TIME ZONE 'UTC') \
             ) \
             SELECT jsonb_build_object( \
                'total', (SELECT count(*) FROM matched), \
                'items', COALESCE((SELECT jsonb_agg(to_jsonb(r) - 'rn' ORDER BY r.rn) FROM ( \
                    SELECT pg.id, pg.title, pg.description, pg.status, pg.priority, pg.created_at, \
                           pg.assigned_to, pg.due_date AT TIME ZONE 'UTC' AS due_date, \
                           pg.archived_at AT TIME ZONE 'UTC' AS archived_at, pg.custom_fields, \
                           pg.project_id, pg.project_name, \
                           row_number() OVER (ORDER BY pg.due_date ASC NULLS LAST, pg.id) AS rn \
                    FROM (SELECT * FROM matched m ORDER BY m.due_date ASC NULLS LAST, m.id \
                          LIMIT $5 OFFSET $6) pg) r), '[]'::jsonb))"
        )
    };
}

/// One page of the tasks visible to a user across every project (MAIR-524), by due date (a task
/// without due date last), then id: the tasks of the projects visible to the user (see
/// `project_visible_to_user_sql!`), filtered by assignee, status and due date. Read with
/// `fetch_one::<PagedRows<VisibleTaskRow>, _>`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetVisibleTasksQueryView {
    params: Vec<QueryParam>,
    assigned: bool,
    archived: bool,
}

impl GetVisibleTasksQueryView {
    pub fn new(user_id: u64, filters: &TaskFilters, limit: u32, offset: u32) -> Self {
        let mut params = vec![
            QueryParam::I32(id_to_sql(user_id)),
            QueryParam::Text(filters.statuses.join(",")),
            // A typed instant and a flag rather than a nullable text: once bound, `NOT $4 OR …`
            // folds away and the due date becomes an index condition.
            QueryParam::DateTime(filters.due_before.unwrap_or(DateTime::UNIX_EPOCH)),
            QueryParam::Bool(filters.due_before.is_some()),
            QueryParam::I64(i64::from(limit)),
            QueryParam::I64(i64::from(offset)),
        ];
        if let Some(assignee) = filters.assigned_to {
            params.push(QueryParam::I32(id_to_sql(assignee)));
        }
        Self {
            params,
            assigned: filters.assigned_to.is_some(),
            archived: filters.includes_archived(),
        }
    }

    pub fn user_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }
}

impl ApiRequestDto for GetVisibleTasksQueryView {
    fn query_sql(&self) -> &'static str {
        match (self.assigned, self.archived) {
            (true, false) => {
                visible_tasks_sql!("t.assigned_to = $7 AND t.archived_at IS NULL AND")
            }
            (true, true) => visible_tasks_sql!("t.assigned_to = $7 AND"),
            (false, false) => visible_tasks_sql!("t.archived_at IS NULL AND"),
            (false, true) => visible_tasks_sql!(""),
        }
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// A task of the cross-project list: the task and its project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisibleTaskRow {
    #[serde(flatten)]
    pub task: Task,
    pub project_id: i32,
    pub project_name: String,
}
