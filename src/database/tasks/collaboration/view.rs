use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// Commentaires et historique libre sont conservés dans `tasks.custom_fields` (`comments`, `history`) ;
// les changements de statut viennent de `task_history` (trigger). Les identifiants d'auteur gardent le
// format public `user-<id>` déjà présent dans les données existantes.

/// Format ISO 8601 en UTC identique à `Date.prototype.toISOString` (millisecondes, suffixe Z).
macro_rules! iso_utc_now_sql {
    () => {
        "to_char(NOW() AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')"
    };
}

/// Collaboration brute d'une tâche : `custom_fields` et changements de statut. Aucune ligne si la tâche
/// n'existe pas dans le projet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTaskCollaborationQueryView {
    params: Vec<QueryParam>,
}

impl GetTaskCollaborationQueryView {
    pub fn new(project_id: u64, task_id: u64) -> Self {
        Self {
            params: vec![
                QueryParam::I32(task_id as i32),
                QueryParam::I32(project_id as i32),
            ],
        }
    }
}

impl ApiRequestDto for GetTaskCollaborationQueryView {
    fn query_sql(&self) -> &'static str {
        "SELECT jsonb_build_object( \
            'custom_fields', COALESCE(t.custom_fields, '{}'::jsonb), \
            'status_changes', COALESCE(( \
                SELECT jsonb_agg(jsonb_build_object( \
                    'id', h.id, \
                    'old_status', h.old_status::text, \
                    'new_status', h.new_status::text, \
                    'changed_at', to_char(h.changed_at, 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'), \
                    'user_id', u.id, \
                    'user_name', NULLIF(concat_ws(' ', u.first_name, u.last_name), '') \
                ) ORDER BY h.changed_at DESC, h.id DESC) \
                FROM task_history h LEFT JOIN users u ON u.id = h.changed_by \
                WHERE h.task_id = t.id), '[]'::jsonb) \
         ) FROM tasks t WHERE t.id = $1 AND t.project_id = $2"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusChangeRow {
    pub id: i32,
    pub old_status: Option<String>,
    pub new_status: Option<String>,
    pub changed_at: String,
    pub user_id: Option<i32>,
    pub user_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCollaborationRow {
    pub custom_fields: serde_json::Value,
    pub status_changes: Vec<StatusChangeRow>,
}

/// Auteur d'un commentaire ou d'une entrée d'historique de tâche.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CollaborationAuthor {
    /// Identifiant de l'auteur, préfixé par `user-`, ou `system` pour une entrée automatique.
    #[schema(example = "user-42")]
    pub id: String,
    /// Nom affiché de l'auteur, ou `Système` pour une entrée automatique.
    #[schema(example = "Jean Dupont")]
    pub name: String,
}

/// Commentaire publié sur une tâche.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct TaskComment {
    /// Identifiant du commentaire.
    #[schema(example = "c-1")]
    pub id: String,
    /// Texte du commentaire.
    #[schema(example = "La réunion publique est calée au 3 octobre.")]
    pub message: String,
    /// Auteur du commentaire, déduit du JWT au moment de l'écriture.
    pub author: CollaborationAuthor,
    /// Date de publication, au format ISO 8601.
    #[serde(rename = "createdAt")]
    #[schema(format = DateTime, example = "2026-09-14T09:12:00Z")]
    pub created_at: String,
}

/// Entrée du journal d'activité d'une tâche.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct TaskHistoryEntry {
    /// Identifiant de l'entrée. Un changement de statut journalisé automatiquement porte un
    /// identifiant préfixé par `status-`.
    #[schema(example = "status-8")]
    pub id: String,
    /// Type d'action. `status_changed` désigne une entrée produite automatiquement.
    #[schema(example = "status_changed")]
    pub action: String,
    /// Libellé lisible de l'action.
    #[schema(example = "Statut modifié : todo → in_progress")]
    pub label: String,
    /// Auteur de l'action, ou `Système` pour une entrée automatique.
    pub author: CollaborationAuthor,
    /// Date de l'action, au format ISO 8601.
    #[serde(rename = "createdAt")]
    #[schema(format = DateTime, example = "2026-09-15T10:04:00Z")]
    pub created_at: String,
    /// Détail des modifications, absent si l'entrée n'en porte pas.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<Object>, example = json!({ "status": { "from": "todo", "to": "in_progress" } }))]
    pub changes: Option<serde_json::Value>,
}

/// Ajoute un commentaire signé par l'utilisateur et le renvoie. Aucune ligne si la tâche n'existe pas
/// dans le projet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddTaskCommentQueryView {
    params: Vec<QueryParam>,
}

impl AddTaskCommentQueryView {
    pub fn new(project_id: u64, task_id: u64, user_id: u64, message: &str) -> Self {
        Self {
            params: vec![
                QueryParam::I32(task_id as i32),
                QueryParam::I32(project_id as i32),
                QueryParam::I32(user_id as i32),
                QueryParam::Text(message.trim().to_string()),
            ],
        }
    }
}

impl ApiRequestDto for AddTaskCommentQueryView {
    fn query_sql(&self) -> &'static str {
        concat!(
            "WITH entry AS ( \
                SELECT jsonb_build_object( \
                    'id', 'comment-' || gen_random_uuid(), \
                    'message', $4::text, \
                    'author', jsonb_build_object( \
                        'id', 'user-' || $3::int, \
                        'name', COALESCE((SELECT NULLIF(concat_ws(' ', first_name, last_name), '') \
                                          FROM users WHERE id = $3), 'Utilisateur ' || $3::int)), \
                    'createdAt', ",
            iso_utc_now_sql!(),
            ") AS value \
             ), updated AS ( \
                UPDATE tasks SET custom_fields = jsonb_set( \
                    COALESCE(custom_fields, '{}'::jsonb), '{comments}', \
                    COALESCE(custom_fields->'comments', '[]'::jsonb) || jsonb_build_array((SELECT value FROM entry)), \
                    true) \
                WHERE id = $1 AND project_id = $2 RETURNING id \
             ) \
             SELECT (SELECT value FROM entry) FROM updated"
        )
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Ajoute une entrée d'historique signée par l'utilisateur et la renvoie. Aucune ligne si la tâche
/// n'existe pas dans le projet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendTaskHistoryQueryView {
    params: Vec<QueryParam>,
}

impl AppendTaskHistoryQueryView {
    pub fn new(
        project_id: u64,
        task_id: u64,
        user_id: u64,
        action: &str,
        label: &str,
        changes: Option<&serde_json::Value>,
    ) -> Self {
        Self {
            params: vec![
                QueryParam::I32(task_id as i32),
                QueryParam::I32(project_id as i32),
                QueryParam::I32(user_id as i32),
                QueryParam::Text(action.to_string()),
                QueryParam::Text(label.to_string()),
                QueryParam::Text(changes.map(|c| c.to_string()).unwrap_or_default()),
            ],
        }
    }
}

impl ApiRequestDto for AppendTaskHistoryQueryView {
    fn query_sql(&self) -> &'static str {
        concat!(
            "WITH entry AS ( \
                SELECT jsonb_strip_nulls(jsonb_build_object( \
                    'id', 'history-' || gen_random_uuid(), \
                    'action', $4::text, \
                    'label', $5::text, \
                    'author', jsonb_build_object( \
                        'id', 'user-' || $3::int, \
                        'name', COALESCE((SELECT NULLIF(concat_ws(' ', first_name, last_name), '') \
                                          FROM users WHERE id = $3), 'Utilisateur ' || $3::int)), \
                    'createdAt', ",
            iso_utc_now_sql!(),
            ", 'changes', NULLIF($6, '')::jsonb)) AS value \
             ), updated AS ( \
                UPDATE tasks SET custom_fields = jsonb_set( \
                    COALESCE(custom_fields, '{}'::jsonb), '{history}', \
                    COALESCE(custom_fields->'history', '[]'::jsonb) || jsonb_build_array((SELECT value FROM entry)), \
                    true) \
                WHERE id = $1 AND project_id = $2 RETURNING id \
             ) \
             SELECT (SELECT value FROM entry) FROM updated"
        )
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
