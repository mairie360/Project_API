use actix_web::http::StatusCode;
use actix_web::{patch, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::project::update_status::view::{ProjectStatus, UpdateProjectStatusQueryView};
use crate::endpoints::v1::projects::access::{require_access, AccessDenied, Requirement};
use crate::endpoints::v1::projects::project_id::ProjectPathParams;

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum PatchMessageError {
    Forbidden,
    NotFound,
    BadRequest,
    DatabaseError,
    UnknownProject,
}

impl std::fmt::Display for PatchMessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PatchMessageError::Forbidden => write!(f, "Forbidden."),
            PatchMessageError::NotFound => write!(f, "Not found."),
            PatchMessageError::BadRequest => {
                write!(f, "Bad request.")
            }
            PatchMessageError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            PatchMessageError::UnknownProject => {
                write!(f, "Unknown project.")
            }
        }
    }
}

impl ResponseError for PatchMessageError {
    fn status_code(&self) -> StatusCode {
        match self {
            PatchMessageError::Forbidden => StatusCode::FORBIDDEN,
            PatchMessageError::NotFound => StatusCode::NOT_FOUND,
            PatchMessageError::BadRequest => StatusCode::BAD_REQUEST,
            PatchMessageError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            PatchMessageError::UnknownProject => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_close_project(
    state: web::Data<AppState>,
    project_id: u64,
) -> Result<(), PatchMessageError> {
    let view = UpdateProjectStatusQueryView::new(project_id, ProjectStatus::Completed);

    state
        .get_smart_db()
        .execute(view)
        .await
        .map_err(|_| PatchMessageError::DatabaseError)?;

    Ok(())
}

#[utoipa::path(
    patch,
    path = "close",
    summary = "Clôturer un projet",
    description = "Bascule le statut du projet à `Completed`. Raccourci sur \
                   `PATCH /api/v1/projects/{project_id}/` avec `{\"status\": \"Completed\"}`, \
                   réservé aux responsables du projet.\n\n\
                   Opération idempotente : clôturer un projet déjà clôturé répond également `200`. \
                   La réponse a un corps vide. Pour rouvrir un projet, repasser par le `PATCH` \
                   avec un autre statut.",
    responses(
        (
            status = 200,
            description = "Projet clôturé, ou déjà clôturé. Corps vide.",
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
#[patch("/close")]
pub async fn close_project(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<ProjectPathParams>,
) -> Result<impl Responder, PatchMessageError> {
    require_access(
        &state,
        auth_user.id,
        params.project_id(),
        None,
        Requirement::ManageProject,
    )
    .await?;
    let project_id = params.project_id();
    trigger_close_project(state, project_id).await?;
    Ok(HttpResponse::Ok())
}

impl From<AccessDenied> for PatchMessageError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => PatchMessageError::NotFound,
            AccessDenied::Forbidden => PatchMessageError::Forbidden,
            AccessDenied::DatabaseError => PatchMessageError::DatabaseError,
        }
    }
}
