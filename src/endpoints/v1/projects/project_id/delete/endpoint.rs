use actix_web::http::StatusCode;
use actix_web::{delete, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::project::delete::view::DeleteProjectQueryView;
use crate::endpoints::v1::projects::access::{require_access, AccessDenied, Requirement};
use crate::endpoints::v1::projects::project_id::ProjectPathParams;

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum DeleteProjectError {
    Forbidden,
    NotFound,
    DatabaseError,
    UnknownProject,
}

impl std::fmt::Display for DeleteProjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeleteProjectError::Forbidden => write!(f, "Forbidden."),
            DeleteProjectError::NotFound => write!(f, "Not found."),
            DeleteProjectError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            DeleteProjectError::UnknownProject => {
                write!(f, "Unknown project.")
            }
        }
    }
}

impl ResponseError for DeleteProjectError {
    fn status_code(&self) -> StatusCode {
        match self {
            DeleteProjectError::Forbidden => StatusCode::FORBIDDEN,
            DeleteProjectError::NotFound => StatusCode::NOT_FOUND,
            DeleteProjectError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            DeleteProjectError::UnknownProject => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_delete_project(
    state: web::Data<AppState>,
    project_id: u64,
) -> Result<(), DeleteProjectError> {
    let view = DeleteProjectQueryView::new(project_id);
    state
        .get_smart_db()
        .execute(view)
        .await
        .map_err(|_| DeleteProjectError::DatabaseError)?;

    Ok(())
}

#[utoipa::path(
    delete,
    path = "",
    summary = "Supprimer un projet",
    description = "Supprime définitivement un projet, ses tâches et les rattachements de ses \
                   membres. Réservé aux responsables du projet. Pour archiver sans détruire, \
                   préférer `PATCH /api/v1/projects/{project_id}/close`.\n\n\
                   Opération idempotente une fois les droits validés : la suppression d'un projet \
                   déjà absent de la base répond également `204`.",
    responses(
        (
            status = 204,
            description = "Projet supprimé. Corps vide.",
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
            description = "Projet inexistant, ou invisible pour l'appelant — les deux cas sont volontairement indiscernables.",
            body = String,
            content_type = "text/plain",
            example = json!("Not found.")
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
        ProjectPathParams
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Projects",
)]
#[delete("/")]
pub async fn delete_project(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<ProjectPathParams>,
) -> Result<impl Responder, DeleteProjectError> {
    require_access(
        &state,
        auth_user.id,
        params.project_id(),
        None,
        Requirement::ManageProject,
    )
    .await?;
    let project_id = params.project_id();
    trigger_delete_project(state, project_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

impl From<AccessDenied> for DeleteProjectError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => DeleteProjectError::NotFound,
            AccessDenied::Forbidden => DeleteProjectError::Forbidden,
            AccessDenied::DatabaseError => DeleteProjectError::DatabaseError,
        }
    }
}
