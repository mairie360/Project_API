use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::tasks::collaboration::view::{
    GetTaskCollaborationQueryView, TaskCollaborationRow,
};
use crate::endpoints::v1::projects::access::{require_access, AccessDenied, Requirement};
use crate::endpoints::v1::projects::project_id::tasks::task_id::collaboration::view::TaskCollaborationView;
use crate::endpoints::v1::projects::project_id::tasks::task_id::TaskPathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum GetTaskCollaborationError {
    Forbidden,
    NotFound,
    DatabaseError,
}

impl std::fmt::Display for GetTaskCollaborationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetTaskCollaborationError::Forbidden => write!(f, "Forbidden."),
            GetTaskCollaborationError::NotFound => write!(f, "Unknown task."),
            GetTaskCollaborationError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for GetTaskCollaborationError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetTaskCollaborationError::Forbidden => StatusCode::FORBIDDEN,
            GetTaskCollaborationError::NotFound => StatusCode::NOT_FOUND,
            GetTaskCollaborationError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_get_task_collaboration(
    state: web::Data<AppState>,
    project_id: u64,
    task_id: u64,
) -> Result<TaskCollaborationView, GetTaskCollaborationError> {
    let rows: Vec<TaskCollaborationRow> = state
        .get_smart_db()
        .fetch_all(&GetTaskCollaborationQueryView::new(project_id, task_id))
        .await
        .map_err(|_| GetTaskCollaborationError::DatabaseError)?;

    rows.into_iter()
        .next()
        .map(Into::into)
        .ok_or(GetTaskCollaborationError::NotFound)
}

#[utoipa::path(
    get,
    path = "collaboration",
    summary = "Consulter les commentaires et l'historique d'une tâche",
    description = "Renvoie en une requête le fil de discussion et le journal d'activité d'une \
                   tâche. Ouvert aux responsables du projet et à l'agent assigné à la tâche.\n\n\
                   Les deux listes sont triées en sens inverse l'une de l'autre : `comments` va du \
                   plus ancien au plus récent (ordre de lecture d'une discussion), `history` du \
                   plus récent au plus ancien (ordre d'un journal).\n\n\
                   `history` mélange deux sources : les entrées ajoutées via \
                   `POST /api/v1/projects/{project_id}/tasks/{task_id}/history`, et les changements \
                   de statut enregistrés automatiquement, reconnaissables à leur `action` valant \
                   `status_changed` et à leur `id` préfixé par `status-`.",
    params(
        TaskPathParams
    ),
    responses(
        (
            status = 200,
            description = "Commentaires et historique de la tâche.",
            body = TaskCollaborationView,
            example = json!({
                "comments": [
                    {
                        "id": "c-1",
                        "message": "La réunion publique est calée au 3 octobre.",
                        "author": { "id": "user-42", "name": "Jean Dupont" },
                        "createdAt": "2026-09-14T09:12:00Z"
                    }
                ],
                "history": [
                    {
                        "id": "status-8",
                        "action": "status_changed",
                        "label": "Statut modifié : todo → in_progress",
                        "author": { "id": "user-42", "name": "Jean Dupont" },
                        "createdAt": "2026-09-15T10:04:00Z",
                        "changes": { "status": { "from": "todo", "to": "in_progress" } }
                    }
                ]
            })
        ),
        (
            status = 400,
            description = "Un segment de l'URL n'est pas un entier, ou le corps JSON est malformé.",
            body = String,
            content_type = "text/plain",
            example = json!("Path deserialize error: can not parse `abc` to a u64")
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
#[get("/collaboration")]
pub async fn get_task_collaboration(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<TaskPathParams>,
) -> Result<impl Responder, GetTaskCollaborationError> {
    require_access(
        &state,
        auth_user.id,
        params.project_id(),
        Some(params.task_id()),
        Requirement::ActOnTask,
    )
    .await?;
    let result =
        trigger_get_task_collaboration(state, params.project_id(), params.task_id()).await?;
    Ok(HttpResponse::Ok().json(result))
}

impl From<AccessDenied> for GetTaskCollaborationError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => GetTaskCollaborationError::NotFound,
            AccessDenied::Forbidden => GetTaskCollaborationError::Forbidden,
            AccessDenied::DatabaseError => GetTaskCollaborationError::DatabaseError,
        }
    }
}
