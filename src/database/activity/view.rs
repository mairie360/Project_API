use mairie360_api_lib::database::db_interface::{id_to_sql, ApiRequestDto, QueryParam};
use serde::{Deserialize, Serialize};

// The project comments are not shown by `GET /projects/`, so adding one invalidates no read
// cache. The timestamps are formatted like the task follow-up (`Date.prototype.toISOString`).

/// One entry of the activity feed of a project: a comment of the project (`kind` = `comment`)
/// or an entry of the history of one of its tasks (`kind` = its `action`: `task_created`,
/// `task_updated`, `status_changed`). `(kind = 'comment', id)` and `(kind <> 'comment', id)`
/// are two id spaces.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActivityRow {
    pub kind: String,
    pub id: i32,
    pub task_id: Option<i32>,
    pub task_title: Option<String>,
    pub author_id: Option<i32>,
    pub author_name: Option<String>,
    pub message: Option<String>,
    pub changes: Option<serde_json::Value>,
    /// Free text of a migrated history entry, `None` otherwise.
    pub label: Option<String>,
    pub old_status: Option<String>,
    pub new_status: Option<String>,
    pub created_at: String,
}

/// One page of the activity of a project, newest first (then the comments before the history,
/// then the highest id first), with the total. Read with `fetch_one::<PagedRows<ActivityRow>, _>`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetProjectActivityQueryView {
    params: Vec<QueryParam>,
}

impl GetProjectActivityQueryView {
    pub fn new(project_id: u64, limit: u32, offset: u32) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(project_id)),
                QueryParam::I64(i64::from(limit)),
                QueryParam::I64(i64::from(offset)),
            ],
        }
    }
}

impl ApiRequestDto for GetProjectActivityQueryView {
    fn query_sql(&self) -> &'static str {
        concat!(
            crate::paged_rows_sql!("$2", "$3"),
            " FROM ( \
                SELECT feed.kind, feed.id, feed.task_id, feed.task_title, feed.author_id, \
                       NULLIF(concat_ws(' ', u.first_name, u.last_name), '') AS author_name, \
                       feed.message, feed.changes, feed.label, feed.old_status, feed.new_status, \
                       to_char(feed.at, 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"') AS created_at, \
                       row_number() OVER (ORDER BY feed.at DESC, feed.rank, feed.id DESC) AS rn \
                FROM ( \
                    SELECT 'comment' AS kind, c.id, NULL::int AS task_id, NULL::text AS task_title, \
                           c.author_id, c.message, NULL::jsonb AS changes, NULL::text AS label, \
                           NULL::text AS old_status, NULL::text AS new_status, \
                           c.created_at AT TIME ZONE 'UTC' AS at, 0 AS rank \
                    FROM project_comments c WHERE c.project_id = $1 \
                    UNION ALL \
                    SELECT h.action::text, h.id, t.id, t.title, h.changed_by, NULL, h.changes, \
                           h.label, h.old_status::text, h.new_status::text, h.changed_at, 1 \
                    FROM task_history h JOIN tasks t ON t.id = h.task_id \
                    WHERE t.project_id = $1 \
                ) feed LEFT JOIN users u ON u.id = feed.author_id \
             ) t"
        )
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Adds a comment to a project, signed by its author, and returns it as an [`ActivityRow`]. Read
/// with `fetch_one::<ActivityRow, _>`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddProjectCommentQueryView {
    params: Vec<QueryParam>,
}

impl AddProjectCommentQueryView {
    pub fn new(project_id: u64, author_id: u64, message: &str) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(project_id)),
                QueryParam::I32(id_to_sql(author_id)),
                QueryParam::Text(message.trim().to_string()),
            ],
        }
    }
}

impl ApiRequestDto for AddProjectCommentQueryView {
    fn query_sql(&self) -> &'static str {
        "WITH c AS ( \
            INSERT INTO project_comments (project_id, author_id, message, created_at) \
            VALUES ($1, $2, $3, NOW()) \
            RETURNING id, author_id, message, created_at \
         ) \
         SELECT to_jsonb(r) FROM ( \
            SELECT 'comment' AS kind, c.id, NULL::int AS task_id, NULL::text AS task_title, \
                   c.author_id, NULLIF(concat_ws(' ', u.first_name, u.last_name), '') AS author_name, \
                   c.message, NULL::jsonb AS changes, NULL::text AS label, \
                   NULL::text AS old_status, NULL::text AS new_status, \
                   to_char(c.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"') \
                       AS created_at \
            FROM c LEFT JOIN users u ON u.id = c.author_id \
         ) r"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
