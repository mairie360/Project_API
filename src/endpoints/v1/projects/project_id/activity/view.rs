use mairie360_api_lib::database::db_interface::id_from_sql;
use utoipa::ToSchema;

use crate::database::activity::view::ActivityRow;
use crate::database::tasks::collaboration::view::{TaskHistoryEntry, TaskHistoryRow};

/// Kind of an activity entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ActivityKind {
    /// A comment posted on the project.
    Comment,
    /// A task of the project was created.
    TaskCreated,
    /// Fields of a task other than its status changed.
    TaskUpdated,
    /// A task moved to another status.
    StatusChanged,
}

impl ActivityKind {
    fn from_stored(value: &str) -> Self {
        match value {
            "comment" => ActivityKind::Comment,
            "task_created" => ActivityKind::TaskCreated,
            "status_changed" => ActivityKind::StatusChanged,
            _ => ActivityKind::TaskUpdated,
        }
    }
}

/// One entry of the activity of a project (MAIR-1089).
#[derive(Debug, Clone, PartialEq, serde::Serialize, ToSchema)]
pub struct ActivityItemView {
    pub kind: ActivityKind,
    /// Id of the comment (`kind` = `comment`) or of the history entry (other kinds): two id
    /// spaces, so `(kind, id)` identifies an entry.
    #[schema(example = 31)]
    pub id: u64,
    /// Task the history entry is about, `null` for a comment.
    #[schema(example = 77)]
    pub task_id: Option<u64>,
    /// Current title of that task, `null` for a comment.
    #[schema(example = "Consult the residents")]
    pub task_title: Option<String>,
    /// Core API id of the author, `null` for a write of the platform itself or an author who no
    /// longer exists.
    #[schema(example = 42)]
    pub author_id: Option<u64>,
    /// First and last name of the author, `null` when unknown.
    #[schema(example = "Jean Dupont")]
    pub author_name: Option<String>,
    /// Text of a comment, `null` for a history entry.
    #[schema(example = "The tender closes on Friday.")]
    pub message: Option<String>,
    /// Changed fields of a history entry as `{"<field>": {"from": …, "to": …}}` (same keys as the
    /// task follow-up, `start_date` included), `null` for a comment and a task creation.
    #[schema(value_type = Option<Object>, example = json!({ "status": { "from": "todo", "to": "in_progress" } }))]
    pub changes: Option<serde_json::Value>,
    /// Human-readable label: `Comment` for a comment, the label of the task follow-up otherwise
    /// (e.g. `Status changed: À faire → En cours`).
    #[schema(example = "Status changed: À faire → En cours")]
    pub label: String,
    /// Date of the entry, ISO 8601 in UTC with milliseconds.
    #[schema(format = DateTime, example = "2026-11-21T09:30:00.000Z")]
    pub created_at: String,
}

impl From<ActivityRow> for ActivityItemView {
    fn from(row: ActivityRow) -> Self {
        let kind = ActivityKind::from_stored(&row.kind);
        let label = if kind == ActivityKind::Comment {
            "Comment".to_string()
        } else {
            TaskHistoryEntry::from(TaskHistoryRow {
                id: row.id,
                action: row.kind.clone(),
                label: row.label.clone(),
                changes: row.changes.clone(),
                old_status: row.old_status.clone(),
                new_status: row.new_status.clone(),
                author_id: row.author_id,
                author_name: row.author_name.clone(),
                created_at: row.created_at.clone(),
            })
            .label
        };
        Self {
            kind,
            id: id_from_sql(row.id),
            task_id: row.task_id.map(id_from_sql),
            task_title: row.task_title,
            author_id: row.author_id.map(id_from_sql),
            author_name: row.author_name,
            message: row.message,
            changes: row.changes,
            label,
            created_at: row.created_at,
        }
    }
}

/// One page of the activity of a project.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetProjectActivityResultView {
    /// Entries of the page, newest first.
    pub items: Vec<ActivityItemView>,
    /// Number of entries of the project (its comments and the history of all its tasks, archived
    /// ones included), whatever the page.
    #[schema(example = 2)]
    pub total: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(kind: &str) -> ActivityRow {
        ActivityRow {
            kind: kind.to_string(),
            id: 8,
            task_id: (kind != "comment").then_some(77),
            task_title: (kind != "comment").then(|| "Consulter".to_string()),
            author_id: Some(42),
            author_name: Some("Jean Dupont".to_string()),
            message: (kind == "comment").then(|| "Vu".to_string()),
            changes: (kind == "task_updated").then(|| {
                serde_json::json!({ "start_date": { "from": null, "to": "2026-10-01T00:00:00" } })
            }),
            label: None,
            old_status: (kind == "status_changed").then(|| "todo".to_string()),
            new_status: (kind == "status_changed").then(|| "in_progress".to_string()),
            created_at: "2026-11-21T09:30:00.000Z".to_string(),
        }
    }

    #[test]
    fn mair_1089_each_kind_gets_its_label() {
        let comment = ActivityItemView::from(row("comment"));
        assert_eq!(
            (comment.kind, comment.label.as_str()),
            (ActivityKind::Comment, "Comment")
        );
        assert_eq!(comment.task_id, None);
        let created = ActivityItemView::from(row("task_created"));
        assert_eq!(created.label, "Task created");
        assert_eq!(created.task_id, Some(77));
        let updated = ActivityItemView::from(row("task_updated"));
        assert_eq!(updated.label, "Task updated: start_date");
        let moved = ActivityItemView::from(row("status_changed"));
        assert_eq!(moved.label, "Status changed: todo → in_progress");
        assert_eq!(
            serde_json::to_value(moved.kind).unwrap(),
            serde_json::json!("status_changed")
        );
    }
}
