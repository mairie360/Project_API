use actix_web::http::StatusCode;
use actix_web::{delete, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::users::remove_user_from_project::view::RemoveUserFromProjectQueryView;
use crate::endpoints::v1::projects::access::{require_access, AccessDenied, Requirement};
use crate::endpoints::v1::projects::project_id::users::user_id::ProjectUserPathParams;

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum RemoveUserFromProjectError {
    Forbidden,
    NotFound,
    DatabaseError,
    UnknownUser,
}

impl std::fmt::Display for RemoveUserFromProjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RemoveUserFromProjectError::Forbidden => write!(f, "Forbidden."),
            RemoveUserFromProjectError::NotFound => write!(f, "Not found."),
            RemoveUserFromProjectError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            RemoveUserFromProjectError::UnknownUser => {
                write!(f, "Unknown user.")
            }
        }
    }
}

impl ResponseError for RemoveUserFromProjectError {
    fn status_code(&self) -> StatusCode {
        match self {
            RemoveUserFromProjectError::Forbidden => StatusCode::FORBIDDEN,
            RemoveUserFromProjectError::NotFound => StatusCode::NOT_FOUND,
            RemoveUserFromProjectError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            RemoveUserFromProjectError::UnknownUser => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_remove_user_from_project(
    state: web::Data<AppState>,
    project_id: u64,
    user_id: u64,
) -> Result<(), RemoveUserFromProjectError> {
    let view = RemoveUserFromProjectQueryView::new(project_id, user_id);
    state
        .get_smart_db()
        .execute(view)
        .await
        .map_err(|_| RemoveUserFromProjectError::DatabaseError)?;

    Ok(())
}

#[utoipa::path(
    delete,
    path = "",
    summary = "Retirer un membre d'un projet",
    description = "Détache un utilisateur du projet. Le compte et le projet sont conservés ; seul \
                   le rattachement disparaît, et le projet cesse d'apparaître dans les \
                   `GET /api/v1/projects/` de cet utilisateur. Réservé aux responsables du \
                   projet.\n\n\
                   Les tâches qui lui étaient assignées ne sont pas réaffectées : elles restent \
                   sur son identifiant. Opération idempotente une fois les droits validés : retirer \
                   quelqu'un qui n'est pas membre répond également `204`.",
    responses(
        (
            status = 204,
            description = "Utilisateur retiré du projet. Corps vide.",
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
        ProjectUserPathParams,
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Users",
)]
#[delete("/")]
pub async fn remove_user_from_project(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<ProjectUserPathParams>,
) -> Result<impl Responder, RemoveUserFromProjectError> {
    require_access(
        &state,
        auth_user.id,
        params.project_id(),
        None,
        Requirement::ManageProject,
    )
    .await?;
    let project_id = params.project_id();
    let user_id = params.user_id();
    trigger_remove_user_from_project(state, project_id, user_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

impl From<AccessDenied> for RemoveUserFromProjectError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => RemoveUserFromProjectError::NotFound,
            AccessDenied::Forbidden => RemoveUserFromProjectError::Forbidden,
            AccessDenied::DatabaseError => RemoveUserFromProjectError::DatabaseError,
        }
    }
}
