use crate::endpoints::validation::{
    check_json, check_label, check_opaque, Validate, ValidationError, MAX_IDENTIFIER_LENGTH,
    MAX_TITLE_LENGTH,
};
use utoipa::ToSchema;

/// Entrée à ajouter au journal d'activité d'une tâche.
#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct AppendTaskHistoryView {
    /// Type d'action. Chaîne libre ; les fronts attendent `task_created`, `task_updated`
    /// ou `status_changed`.
    #[schema(max_length = 64, example = "task_updated")]
    pub action: String,
    /// Libellé lisible de l'action, affiché tel quel dans le journal.
    #[schema(
        min_length = 1,
        max_length = 255,
        example = "Budget révisé après consultation"
    )]
    pub label: String,
    /// Détail libre des modifications, de forme quelconque.
    #[schema(value_type = Option<Object>, example = json!({ "budget": { "from": 12000, "to": 15500 } }))]
    pub changes: Option<serde_json::Value>,
}

/// Actions que les clients peuvent consigner ; l'auteur est toujours l'appelant.
pub const ALLOWED_HISTORY_ACTIONS: [&str; 3] = ["task_created", "task_updated", "status_changed"];

impl Validate for AppendTaskHistoryView {
    fn validate(&self) -> Result<(), ValidationError> {
        check_opaque("action", &self.action, MAX_IDENTIFIER_LENGTH)?;
        check_label("label", &self.label, MAX_TITLE_LENGTH)?;
        self.changes
            .as_ref()
            .map_or(Ok(()), |changes| check_json("changes", changes))
    }
}
