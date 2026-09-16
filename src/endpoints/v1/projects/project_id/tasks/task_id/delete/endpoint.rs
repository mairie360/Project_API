use actix_web::http::StatusCode;
use actix_web::{delete, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::tasks::delete_task::view::DeleteTaskQueryView;
use crate::endpoints::v1::projects::access::{require_access, AccessDenied, Requirement};
use crate::endpoints::v1::projects::project_id::tasks::task_id::TaskPathParams;

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum DeleteTaskError {
    Forbidden,
    NotFound,
    DatabaseError,
    UnknownTask,
}

impl std::fmt::Display for DeleteTaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeleteTaskError::Forbidden => write!(f, "Forbidden."),
            DeleteTaskError::NotFound => write!(f, "Not found."),
            DeleteTaskError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            DeleteTaskError::UnknownTask => {
                write!(f, "Unknown task.")
            }
        }
    }
}

impl ResponseError for DeleteTaskError {
    fn status_code(&self) -> StatusCode {
        match self {
            DeleteTaskError::Forbidden => StatusCode::FORBIDDEN,
            DeleteTaskError::NotFound => StatusCode::NOT_FOUND,
            DeleteTaskError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            DeleteTaskError::UnknownTask => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_delete_task(
    state: web::Data<AppState>,
    _project_id: u64,
    task_id: u64,
) -> Result<(), DeleteTaskError> {
    let view = DeleteTaskQueryView::new(task_id);
    state
        .get_smart_db()
        .execute(view)
        .await
        .map_err(|_| DeleteTaskError::DatabaseError)?;

    Ok(())
}

#[utoipa::path(
    delete,
    path = "",
    summary = "Supprimer une tâche",
    description = "Supprime définitivement une tâche, avec ses commentaires et son historique. \
                   Réservé aux responsables du projet : un agent assigné à la tâche ne peut pas la \
                   supprimer, seulement changer son statut.\n\n\
                   Opération idempotente une fois les droits validés : supprimer une tâche déjà \
                   absente de la base répond également `204`.",
    responses(
        (
            status = 204,
            description = "Tâche supprimée. Corps vide.",
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
            description = "Projet visible par l'appelant, mais droits insuffisants pour cette opération.",
            body = String,
            content_type = "text/plain",
            example = json!("Forbidden.")
        ),
        (
            status = 404,
            description = "Projet ou tâche inexistant, ou projet invisible pour l'appelant. Une tâche appartenant à un autre projet est traitée comme absente.",
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
    params(
        TaskPathParams
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Tasks",
)]
#[delete("/")]
pub async fn delete_task(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<TaskPathParams>,
) -> Result<impl Responder, DeleteTaskError> {
    require_access(
        &state,
        auth_user.id,
        params.project_id(),
        Some(params.task_id()),
        Requirement::ManageProject,
    )
    .await?;
    trigger_delete_task(state, params.project_id(), params.task_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

impl From<AccessDenied> for DeleteTaskError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => DeleteTaskError::NotFound,
            AccessDenied::Forbidden => DeleteTaskError::Forbidden,
            AccessDenied::DatabaseError => DeleteTaskError::DatabaseError,
        }
    }
}
