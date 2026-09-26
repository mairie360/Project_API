use actix_web::http::StatusCode;
use actix_web::{post, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::tasks::collaboration::view::{AppendTaskHistoryQueryView, TaskHistoryEntry};
use crate::endpoints::v1::projects::access::{require_access, AccessDenied, Requirement};
use crate::endpoints::v1::projects::project_id::tasks::task_id::history::view::{
    AppendTaskHistoryView, ALLOWED_HISTORY_ACTIONS,
};
use crate::endpoints::v1::projects::project_id::tasks::task_id::TaskPathParams;
use crate::endpoints::validation::ValidatedJson;

#[derive(Debug, Clone, PartialEq)]
pub enum AppendTaskHistoryError {
    Forbidden,
    BadRequest,
    NotFound,
    DatabaseError,
}

impl std::fmt::Display for AppendTaskHistoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppendTaskHistoryError::Forbidden => write!(f, "Forbidden."),
            AppendTaskHistoryError::BadRequest => write!(f, "Bad request."),
            AppendTaskHistoryError::NotFound => write!(f, "Unknown task."),
            AppendTaskHistoryError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for AppendTaskHistoryError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppendTaskHistoryError::Forbidden => StatusCode::FORBIDDEN,
            AppendTaskHistoryError::BadRequest => StatusCode::BAD_REQUEST,
            AppendTaskHistoryError::NotFound => StatusCode::NOT_FOUND,
            AppendTaskHistoryError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_append_task_history(
    state: web::Data<AppState>,
    params: &TaskPathParams,
    user_id: u64,
    view: AppendTaskHistoryView,
) -> Result<TaskHistoryEntry, AppendTaskHistoryError> {
    if !ALLOWED_HISTORY_ACTIONS.contains(&view.action.trim()) || view.label.trim().is_empty() {
        return Err(AppendTaskHistoryError::BadRequest);
    }
    if view
        .changes
        .as_ref()
        .is_some_and(|changes| !changes.is_object())
    {
        return Err(AppendTaskHistoryError::BadRequest);
    }

    let entries: Vec<TaskHistoryEntry> = state
        .get_smart_db()
        .fetch_all(&AppendTaskHistoryQueryView::new(
            params.project_id(),
            params.task_id(),
            user_id,
            view.action.trim(),
            view.label.trim(),
            view.changes.as_ref(),
        ))
        .await
        .map_err(|_| AppendTaskHistoryError::DatabaseError)?;

    entries
        .into_iter()
        .next()
        .ok_or(AppendTaskHistoryError::NotFound)
}

#[utoipa::path(
    post,
    path = "history",
    summary = "Ajouter une entrée d'historique à une tâche",
    description = "Ajoute une entrée au journal d'activité d'une tâche. Ouvert aux responsables du \
                   projet et à l'agent assigné à la tâche.\n\n\
                   Sert à tracer une action métier que l'API ne détecte pas d'elle-même. Les \
                   changements de statut, eux, sont déjà journalisés automatiquement par \
                   `PATCH …/tasks/{task_id}/` : les redéclarer ici créerait un doublon.\n\n\
                   `action` est une chaîne libre ; les valeurs attendues par les fronts sont \
                   `task_created`, `task_updated` et `status_changed`. L'auteur est déduit du JWT.",
    params(
        TaskPathParams
    ),
    request_body(
        content = AppendTaskHistoryView,
        description = "Entrée à journaliser.",
        example = json!({
            "action": "task_updated",
            "label": "Budget révisé après consultation",
            "changes": { "budget": { "from": 12000, "to": 15500 } }
        })
    ),
    responses(
        (
            status = 201,
            description = "Entrée ajoutée à l'historique.",
            body = TaskHistoryEntry,
            example = json!({
                "id": "h-4",
                "action": "task_updated",
                "label": "Budget révisé après consultation",
                "author": { "id": "user-42", "name": "Jean Dupont" },
                "createdAt": "2026-09-16T11:30:00Z",
                "changes": { "budget": { "from": 12000, "to": 15500 } }
            })
        ),
        (
            status = 400,
            description = "Malformed JSON body, URL segment not an integer, missing or empty field, `action` not allowed or longer than 64 characters, `label` longer than 255 characters or containing `<` / `>` / control characters, or a `changes` string or key containing `<`, `>` or control characters.",
            body = String,
            content_type = "text/plain",
            example = json!("Invalid `label`: must not contain `<` or `>`")
        ),
        (
            status = 401,
            description = "En-tête `Authorization` absent, JWT invalide ou expiré, ou session révoquée.",
            body = String,
            content_type = "text/plain",
            example = json!("Jeton expiré")
        ),
        (
            status = 403,
            description = "Ni responsable du projet, ni agent assigné à la tâche.",
            body = String,
            content_type = "text/plain",
            example = json!("Forbidden.")
        ),
        (
            status = 404,
            description = "Projet ou tâche inexistant, ou projet invisible pour l'appelant.",
            body = String,
            content_type = "text/plain",
            example = json!("Unknown task.")
        ),
        (
            status = 500,
            description = "Erreur de base de données.",
            body = String,
            content_type = "text/plain",
            example = json!("An error occurred while accessing the database.")
        ),
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Tasks",
)]
#[post("/history")]
pub async fn append_task_history(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<TaskPathParams>,
    view: ValidatedJson<AppendTaskHistoryView>,
) -> Result<impl Responder, AppendTaskHistoryError> {
    require_access(
        &state,
        auth_user.id,
        params.project_id(),
        Some(params.task_id()),
        Requirement::ActOnTask,
    )
    .await?;
    let entry =
        trigger_append_task_history(state, &params, auth_user.id, view.into_inner()).await?;
    Ok(HttpResponse::Created().json(entry))
}

impl From<AccessDenied> for AppendTaskHistoryError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => AppendTaskHistoryError::NotFound,
            AccessDenied::Forbidden => AppendTaskHistoryError::Forbidden,
            AccessDenied::DatabaseError => AppendTaskHistoryError::DatabaseError,
        }
    }
}
