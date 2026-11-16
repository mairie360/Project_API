use actix_web::http::StatusCode;
use actix_web::{patch, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::smart_db::SmartTransaction;
use mairie360_api_lib::state::AppState;

use crate::endpoints::db_error::log_db_error;

use crate::database::project::update_status::view::{ProjectStatus, UpdateProjectStatusQueryView};
use crate::endpoints::v1::projects::access::{begin_write, commit, AccessDenied, Requirement};
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
    tx: &mut SmartTransaction,
    project_id: u64,
) -> Result<(), PatchMessageError> {
    let view = UpdateProjectStatusQueryView::new(project_id, ProjectStatus::Completed);

    tx.execute(&view).await.map_err(|e| {
        log_db_error("projects/project_id/close", &e);
        PatchMessageError::DatabaseError
    })?;

    Ok(())
}

#[utoipa::path(
    patch,
    path = "close",
    summary = "Close a project",
    description = "Sets the project status to `Completed`. Shortcut for \
               `PATCH /api/v1/projects/{project_id}/` with `{\"status\": \"Completed\"}`, \
               reserved to the managers of the project.\n\n\
               Idempotent: closing a project already closed also answers `200`. \
               The answer has an empty body. To reopen a project, use the `PATCH` \
               with another status.",
    responses(
        (
            status = 200,
            description = "Project closed, or already closed. Empty body.",
        ),
        (
            status = 400,
            description = "A URL segment is not an integer, or the JSON body is malformed.",
            body = String,
            content_type = "text/plain",
            example = json!("Path deserialize error: can not parse `abc` to a u64")
        ),
        (
            status = 401,
            description = "Missing `Authorization` header, invalid or expired JWT, or revoked session.",
            body = String,
            content_type = "text/plain",
            example = json!("Jeton expiré")
        ),
        (
            status = 403,
            description = "Project visible to the caller, but insufficient rights for this operation.",
            body = String,
            content_type = "text/plain",
            example = json!("Forbidden.")
        ),
        (
            status = 404,
            description = "Project that does not exist, or that the caller cannot see — the two cases are indistinguishable on purpose.",
            body = String,
            content_type = "text/plain",
            example = json!("Not found.")
        ),
        (
            status = 500,
            description = "Database error.",
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
    let (mut tx, _) = begin_write(
        &state,
        auth_user.id,
        params.project_id(),
        None,
        Requirement::ManageProject,
    )
    .await?;
    let project_id = params.project_id();
    trigger_close_project(&mut tx, project_id).await?;
    commit(tx, "projects/project_id/close").await?;
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
