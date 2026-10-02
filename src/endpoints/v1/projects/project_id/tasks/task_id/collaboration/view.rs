use utoipa::ToSchema;

use crate::database::tasks::collaboration::view::{
    TaskCollaborationRow, TaskComment, TaskHistoryEntry,
};

/// Collaboration thread of a task: one page of its comments and one page of its history.
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct TaskCollaborationView {
    /// Comments, oldest first (reading order of a discussion), paginated by `limit` / `offset`.
    pub comments: Vec<TaskComment>,
    /// Total number of comments on the task, whatever the page.
    #[schema(example = 1)]
    pub comments_total: u64,
    /// History entries, newest first (order of a log), paginated by the same `limit` / `offset`.
    pub history: Vec<TaskHistoryEntry>,
    /// Total number of history entries of the task, whatever the page.
    #[schema(example = 2)]
    pub history_total: u64,
}

impl From<TaskCollaborationRow> for TaskCollaborationView {
    fn from(row: TaskCollaborationRow) -> Self {
        Self {
            comments: row.comments.items.into_iter().map(|c| c.value).collect(),
            comments_total: u64::try_from(row.comments.total).unwrap_or(0),
            history: row.history.items.into_iter().map(Into::into).collect(),
            history_total: u64::try_from(row.history.total).unwrap_or(0),
        }
    }
}
