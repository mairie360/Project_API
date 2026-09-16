use utoipa::ToSchema;

use crate::database::tasks::collaboration::view::{
    CollaborationAuthor, StatusChangeRow, TaskCollaborationRow, TaskComment, TaskHistoryEntry,
};

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct TaskCollaborationView {
    /// Commentaires, du plus ancien au plus récent.
    pub comments: Vec<TaskComment>,
    /// Historique libre et changements de statut, du plus récent au plus ancien.
    pub history: Vec<TaskHistoryEntry>,
}

fn entries<T: serde::de::DeserializeOwned>(custom_fields: &serde_json::Value, key: &str) -> Vec<T> {
    custom_fields
        .get(key)
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|value| serde_json::from_value(value.clone()).ok())
        .collect()
}

fn status_change_entry(change: StatusChangeRow) -> TaskHistoryEntry {
    let from = change.old_status.clone();
    let to = change.new_status.clone();
    TaskHistoryEntry {
        id: format!("status-{}", change.id),
        action: "status_changed".to_string(),
        label: format!(
            "Statut modifié : {} → {}",
            from.as_deref().unwrap_or("inconnu"),
            to.as_deref().unwrap_or("inconnu")
        ),
        author: CollaborationAuthor {
            id: change
                .user_id
                .map(|id| format!("user-{id}"))
                .unwrap_or_else(|| "system".to_string()),
            name: change.user_name.unwrap_or_else(|| "Système".to_string()),
        },
        created_at: change.changed_at,
        changes: Some(serde_json::json!({ "status": { "from": from, "to": to } })),
    }
}

impl From<TaskCollaborationRow> for TaskCollaborationView {
    fn from(row: TaskCollaborationRow) -> Self {
        let mut comments: Vec<TaskComment> = entries(&row.custom_fields, "comments");
        comments.sort_by(|a, b| a.created_at.cmp(&b.created_at));

        let mut history: Vec<TaskHistoryEntry> = entries(&row.custom_fields, "history");
        history.extend(row.status_changes.into_iter().map(status_change_entry));
        history.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Self { comments, history }
    }
}
