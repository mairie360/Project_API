use actix_web::http::StatusCode;
use actix_web::{patch, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::project::update::view::UpdateProjectQueryView;
use crate::database::project::update_status::view::ProjectStatus as DbProjectStatus;
use crate::endpoints::v1::projects::access::{require_access, AccessDenied, Requirement};
use crate::endpoints::v1::projects::get::view::ProjectStatus;
use crate::endpoints::v1::projects::project_id::patch::view::UpdateProjectView;
use crate::endpoints::v1::projects::project_id::ProjectPathParams;
use crate::endpoints::validation::ValidatedJson;

#[derive(Debug, Clone, PartialEq)]
pub enum UpdateProjectError {
    Forbidden,
    BadRequest,
    NotFound,
    DatabaseError,
}

impl std::fmt::Display for UpdateProjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdateProjectError::Forbidden => write!(f, "Forbidden."),
            UpdateProjectError::BadRequest => write!(f, "Bad request."),
            UpdateProjectError::NotFound => write!(f, "Unknown project."),
            UpdateProjectError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for UpdateProjectError {
    fn status_code(&self) -> StatusCode {
        match self {
            UpdateProjectError::Forbidden => StatusCode::FORBIDDEN,
            UpdateProjectError::BadRequest => StatusCode::BAD_REQUEST,
            UpdateProjectError::NotFound => StatusCode::NOT_FOUND,
            UpdateProjectError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_update_project(
    state: web::Data<AppState>,
    project_id: u64,
    view: UpdateProjectView,
) -> Result<(), UpdateProjectError> {
    if view
        .name
        .as_deref()
        .is_some_and(|name| name.trim().is_empty())
    {
        return Err(UpdateProjectError::BadRequest);
    }
    let status = match view.status {
        Some(ProjectStatus::Error) => return Err(UpdateProjectError::BadRequest),
        Some(status) => Some(DbProjectStatus::from(status.to_string())),
        None => None,
    };

    let updated: bool = state
        .get_smart_db()
        .fetch_scalar(&UpdateProjectQueryView::new(
            project_id,
            view.name.as_deref().map(str::trim),
            view.description.as_deref(),
            status,
        ))
        .await
        .map_err(|_| UpdateProjectError::DatabaseError)?;

    if updated {
        Ok(())
    } else {
        Err(UpdateProjectError::NotFound)
    }
}

#[utoipa::path(
    patch,
    params(
        ProjectPathParams,
    ),
    path = "",
    summary = "Modifier un projet",
    description = "Met à jour le nom, la description ou le statut d'un projet. Modification \
                   partielle : un champ absent ou `null` reste inchangé. Réservé aux responsables \
                   du projet.\n\n\
                   Un `name` réduit à des espaces est refusé en `400`, de même que le statut \
                   `Error`, qui n'est qu'une valeur de repli à la lecture et n'est pas \
                   assignable.\n\n\
                   La réponse a un corps vide ; relire le projet avec \
                   `GET /api/v1/projects/{project_id}/`.",
    request_body(
        content = UpdateProjectView,
        description = "Champs à modifier. Tous facultatifs.",
        example = json!({ "status": "Suspended" })
    ),
    responses(
        (
            status = 204,
            description = "Projet mis à jour. Corps vide.",
        ),
        (
            status = 400,
            description = "Malformed JSON body, `project_id` not an integer, `name` empty, `Error` status, or a field breaking its rules: `name` 1 to 255 characters, not blank, no control character, no `<` or `>`; `description` at most 5000 characters, no `<` or `>`, no control character other than line breaks and tabs.",
            body = String,
            content_type = "text/plain",
            example = json!("Invalid `name`: must be at most 255 characters")
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
            description = "Projet inexistant, ou invisible pour l'appelant.",
            body = String,
            content_type = "text/plain",
            example = json!("Unknown project.")
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
    tag = "Projects",
)]
#[patch("/")]
pub async fn update_project(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<ProjectPathParams>,
    view: ValidatedJson<UpdateProjectView>,
) -> Result<impl Responder, UpdateProjectError> {
    require_access(
        &state,
        auth_user.id,
        params.project_id(),
        None,
        Requirement::ManageProject,
    )
    .await?;
    trigger_update_project(state, params.project_id(), view.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}

impl From<AccessDenied> for UpdateProjectError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => UpdateProjectError::NotFound,
            AccessDenied::Forbidden => UpdateProjectError::Forbidden,
            AccessDenied::DatabaseError => UpdateProjectError::DatabaseError,
        }
    }
}
