use utoipa::ToSchema;

#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct AppendTaskHistoryView {
    /// Type d'action : `task_created`, `task_updated` ou `status_changed`.
    pub action: String,
    /// Libellé lisible de l'action.
    pub label: String,
    /// Détail libre des modifications.
    #[schema(value_type = Option<Object>)]
    pub changes: Option<serde_json::Value>,
}

/// Actions que les clients peuvent consigner ; l'auteur est toujours l'appelant.
pub const ALLOWED_HISTORY_ACTIONS: [&str; 3] = ["task_created", "task_updated", "status_changed"];
