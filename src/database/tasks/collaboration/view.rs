use mairie360_api_lib::database::db_interface::{id_to_sql, ApiRequestDto, QueryParam};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::database::paged::PagedRows;

// Comments live in `task_comments`, the history in `task_history`, which only the database trigger
// `fn_log_task_change` writes (task creation, field changes, status changes, signed with
// `tasks.updated_by`). Author ids keep the public `user-<id>` format.

/// ISO 8601 UTC format identical to `Date.prototype.toISOString` (milliseconds, `Z` suffix), for a
/// TIMESTAMP column stored in UTC.
macro_rules! iso_utc_sql {
    ($column:literal) => {
        concat!(
            "to_char(",
            $column,
            ", 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')"
        )
    };
}

/// Public JSON of a comment row `c` joined with its author `u`.
macro_rules! comment_json_sql {
    () => {
        concat!(
            "jsonb_build_object( \
                'id', 'comment-' || c.id, \
                'message', c.message, \
                'author', CASE WHEN c.author_id IS NULL \
                    THEN jsonb_build_object('id', 'system', 'name', 'System') \
                    ELSE jsonb_build_object('id', 'user-' || c.author_id, \
                        'name', COALESCE(NULLIF(concat_ws(' ', u.first_name, u.last_name), ''), \
                                         'User ' || c.author_id)) END, \
                'createdAt', ",
            iso_utc_sql!("c.created_at"),
            ")"
        )
    };
}

/// One page of the comments (oldest first) and of the history (newest first) of a task. No row if the
/// task is not in the project. Read with `fetch_all::<TaskCollaborationRow, _>`.
// One list of the follow-up: its total and the rows of the page only, numbered in their order. Only
// the rows of the page are turned into JSON (MAIR-502: every row of a 1 000-comment feed used to be).
// `$order` orders the feed (`$page_order`: the same, on `page_rows`), `$columns` are the columns of a
// row of the page, read from `page_rows`.
macro_rules! feed_page_sql {
    ($table:literal, $order:literal, $page_order:literal, $columns:expr) => {
        concat!(
            "jsonb_build_object('total', (SELECT count(*) FROM ",
            $table,
            " WHERE task_id = task.id), 'items', COALESCE((SELECT jsonb_agg(to_jsonb(r) - 'rn' ORDER BY r.rn) \
                FROM (SELECT ",
            $columns,
            ", row_number() OVER (ORDER BY ",
            $page_order,
            ") AS rn FROM (SELECT * FROM ",
            $table,
            " WHERE task_id = task.id ORDER BY ",
            $order,
            " LIMIT $3 OFFSET $4) page_rows LEFT JOIN users u ON u.id = page_rows.author_ref) r), '[]'::jsonb))"
        )
    };
}

macro_rules! comment_columns_sql {
    () => {
        concat!(
            "jsonb_build_object( \
                'id', 'comment-' || page_rows.id, \
                'message', page_rows.message, \
                'author', CASE WHEN page_rows.author_id IS NULL \
                    THEN jsonb_build_object('id', 'system', 'name', 'System') \
                    ELSE jsonb_build_object('id', 'user-' || page_rows.author_id, \
                        'name', COALESCE(NULLIF(concat_ws(' ', u.first_name, u.last_name), ''), \
                                         'User ' || page_rows.author_id)) END, \
                'createdAt', ",
            iso_utc_sql!("page_rows.created_at"),
            ") AS value"
        )
    };
}

macro_rules! history_columns_sql {
    () => {
        concat!(
            "page_rows.id, page_rows.action, page_rows.label, page_rows.changes, \
             page_rows.old_status::text AS old_status, page_rows.new_status::text AS new_status, \
             page_rows.changed_by AS author_id, \
             NULLIF(concat_ws(' ', u.first_name, u.last_name), '') AS author_name, ",
            iso_utc_sql!("page_rows.changed_at"),
            " AS created_at"
        )
    };
}

/// One page of the comments (oldest first, or the most recent first with `latest_comments`) and one
/// page of the history (newest first) of a task, with their totals. Read with `fetch_all::<TaskCollaborationRow, _>`;
/// no row when the task is not in the project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTaskCollaborationQueryView {
    params: Vec<QueryParam>,
    latest_comments: bool,
}

impl GetTaskCollaborationQueryView {
    pub fn new(project_id: u64, task_id: u64, limit: u32, offset: u32) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(task_id)),
                QueryParam::I32(id_to_sql(project_id)),
                QueryParam::I64(i64::from(limit)),
                QueryParam::I64(i64::from(offset)),
            ],
            latest_comments: false,
        }
    }

    /// Pages the comments from the most recent one (MAIR-502), like the history.
    #[must_use]
    pub fn latest_comments_first(mut self) -> Self {
        self.latest_comments = true;
        self
    }
}

impl ApiRequestDto for GetTaskCollaborationQueryView {
    fn query_sql(&self) -> &'static str {
        // `author_ref` names the author column of each table for the join with `users`.
        if self.latest_comments {
            concat!(
                "SELECT jsonb_build_object('comments', ",
                feed_page_sql!(
                    "(SELECT c.*, c.author_id AS author_ref FROM task_comments c) comments",
                    "created_at DESC, id DESC",
                    "page_rows.created_at DESC, page_rows.id DESC",
                    comment_columns_sql!()
                ),
                ", 'history', ",
                feed_page_sql!(
                    "(SELECT h.*, h.changed_by AS author_ref FROM task_history h) history",
                    "changed_at DESC, id DESC",
                    "page_rows.changed_at DESC, page_rows.id DESC",
                    history_columns_sql!()
                ),
                ") FROM tasks task WHERE task.id = $1 AND task.project_id = $2"
            )
        } else {
            concat!(
                "SELECT jsonb_build_object('comments', ",
                feed_page_sql!(
                    "(SELECT c.*, c.author_id AS author_ref FROM task_comments c) comments",
                    "created_at, id",
                    "page_rows.created_at, page_rows.id",
                    comment_columns_sql!()
                ),
                ", 'history', ",
                feed_page_sql!(
                    "(SELECT h.*, h.changed_by AS author_ref FROM task_history h) history",
                    "changed_at DESC, id DESC",
                    "page_rows.changed_at DESC, page_rows.id DESC",
                    history_columns_sql!()
                ),
                ") FROM tasks task WHERE task.id = $1 AND task.project_id = $2"
            )
        }
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentRow {
    pub value: TaskComment,
}

/// History row of [`GetTaskCollaborationQueryView`], turned into a [`TaskHistoryEntry`] by the endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskHistoryRow {
    pub id: i32,
    pub action: String,
    /// Free text of an entry migrated from the former client-written history; `None` otherwise.
    pub label: Option<String>,
    pub changes: Option<serde_json::Value>,
    pub old_status: Option<String>,
    pub new_status: Option<String>,
    pub author_id: Option<i32>,
    pub author_name: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCollaborationRow {
    pub comments: PagedRows<CommentRow>,
    pub history: PagedRows<TaskHistoryRow>,
}

/// Author of a comment or of a history entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CollaborationAuthor {
    /// Author id, prefixed with `user-`, or `system` for a write made by the platform itself (e.g. the
    /// unassignment of an archived agent) or by an author who no longer exists.
    #[schema(example = "user-42")]
    pub id: String,
    /// Display name of the author, or `System`.
    #[schema(example = "Jean Dupont")]
    pub name: String,
}

/// Comment posted on a task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct TaskComment {
    /// Comment id, `comment-` followed by a number.
    #[schema(example = "comment-31")]
    pub id: String,
    /// Text of the comment.
    #[schema(example = "La réunion publique est calée au 3 octobre.")]
    pub message: String,
    /// Author of the comment, taken from the JWT when it was written.
    pub author: CollaborationAuthor,
    /// Publication date, ISO 8601 in UTC.
    #[serde(rename = "createdAt")]
    #[schema(format = DateTime, example = "2026-09-14T09:12:00.000Z")]
    pub created_at: String,
}

/// Entry of the activity log of a task. Every entry is written by the server, never by a client.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct TaskHistoryEntry {
    /// Entry id, `history-` followed by a number.
    #[schema(example = "history-8")]
    pub id: String,
    /// Kind of action: `task_created`, `task_updated` (fields other than the status) or
    /// `status_changed`.
    #[schema(example = "status_changed")]
    pub action: String,
    /// Human-readable label, generated from `action` and `changes`.
    #[schema(example = "Status changed: todo → in_progress")]
    pub label: String,
    /// Author of the change, or `System`.
    pub author: CollaborationAuthor,
    /// Date of the change, ISO 8601 in UTC.
    #[serde(rename = "createdAt")]
    #[schema(format = DateTime, example = "2026-09-15T10:04:00.000Z")]
    pub created_at: String,
    /// Changed fields as `{"<field>": {"from": …, "to": …}}`. Keys are `status`, `title`, `description`,
    /// `priority`, `due_date`, `assigned_to` and `fields`. Absent on `task_created`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<Object>, example = json!({ "status": { "from": "todo", "to": "in_progress" } }))]
    pub changes: Option<serde_json::Value>,
}

/// Order in which changed fields are listed in a `task_updated` label.
const TRACKED_FIELDS: [&str; 6] = [
    "title",
    "description",
    "priority",
    "due_date",
    "assigned_to",
    "fields",
];

impl From<TaskHistoryRow> for TaskHistoryEntry {
    fn from(row: TaskHistoryRow) -> Self {
        let label = row
            .label
            .clone()
            .unwrap_or_else(|| match row.action.as_str() {
                "task_created" => "Task created".to_string(),
                "status_changed" => format!(
                    "Status changed: {} → {}",
                    row.old_status.as_deref().unwrap_or("unknown"),
                    row.new_status.as_deref().unwrap_or("unknown")
                ),
                _ => {
                    let changed: Vec<&str> = TRACKED_FIELDS
                        .into_iter()
                        .filter(|field| {
                            row.changes
                                .as_ref()
                                .is_some_and(|changes| changes.get(field).is_some())
                        })
                        .collect();
                    if changed.is_empty() {
                        "Task updated".to_string()
                    } else {
                        format!("Task updated: {}", changed.join(", "))
                    }
                }
            });
        TaskHistoryEntry {
            id: format!("history-{}", row.id),
            action: row.action,
            label,
            author: match row.author_id {
                Some(id) => CollaborationAuthor {
                    id: format!("user-{id}"),
                    name: row.author_name.unwrap_or_else(|| format!("User {id}")),
                },
                None => CollaborationAuthor {
                    id: "system".to_string(),
                    name: "System".to_string(),
                },
            },
            created_at: row.created_at,
            changes: row.changes,
        }
    }
}

/// Adds a comment signed by the user and returns it. No row if the task is not in the project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddTaskCommentQueryView {
    params: Vec<QueryParam>,
}

impl AddTaskCommentQueryView {
    pub fn new(project_id: u64, task_id: u64, user_id: u64, message: &str) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(task_id)),
                QueryParam::I32(id_to_sql(project_id)),
                QueryParam::I32(id_to_sql(user_id)),
                QueryParam::Text(message.trim().to_string()),
            ],
        }
    }
}

impl ApiRequestDto for AddTaskCommentQueryView {
    fn query_sql(&self) -> &'static str {
        concat!(
            "WITH c AS ( \
                INSERT INTO task_comments (task_id, author_id, message, created_at) \
                SELECT id, $3, $4, NOW() AT TIME ZONE 'UTC' FROM tasks WHERE id = $1 AND project_id = $2 \
                RETURNING id, author_id, message, created_at \
             ) \
             SELECT ",
            comment_json_sql!(),
            " FROM c LEFT JOIN users u ON u.id = c.author_id"
        )
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
