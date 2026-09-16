use utoipa::ToSchema;

/// Entrée à ajouter au journal d'activité d'une tâche.
#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct AppendTaskHistoryView {
    /// Type d'action. Chaîne libre ; les fronts attendent `task_created`, `task_updated`
    /// ou `status_changed`.
    #[schema(example = "task_updated")]
    pub action: String,
    /// Libellé lisible de l'action, affiché tel quel dans le journal.
    #[schema(example = "Budget révisé après consultation")]
    pub label: String,
    /// Détail libre des modifications, de forme quelconque.
    #[schema(value_type = Option<Object>, example = json!({ "budget": { "from": 12000, "to": 15500 } }))]
    pub changes: Option<serde_json::Value>,
}

/// Actions que les clients peuvent consigner ; l'auteur est toujours l'appelant.
pub const ALLOWED_HISTORY_ACTIONS: [&str; 3] = ["task_created", "task_updated", "status_changed"];
