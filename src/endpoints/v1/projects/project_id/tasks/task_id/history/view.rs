use utoipa::ToSchema;

#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct AppendTaskHistoryView {
    /// Type d'action (ex. `task_updated`).
    pub action: String,
    /// Libellé lisible de l'action.
    pub label: String,
    /// Détail libre des modifications.
    #[schema(value_type = Option<Object>)]
    pub changes: Option<serde_json::Value>,
}
