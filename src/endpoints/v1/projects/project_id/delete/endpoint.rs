use actix_web::http::StatusCode;
use actix_web::{delete, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::smart_db::SmartTransaction;
use mairie360_api_lib::state::AppState;

use crate::endpoints::db_error::log_db_error;

use crate::database::project::delete::view::DeleteProjectQueryView;
use crate::endpoints::v1::projects::access::{begin_write, commit, AccessDenied, Requirement};
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
    tx: &mut SmartTransaction,
    project_id: u64,
) -> Result<(), DeleteProjectError> {
    let view = DeleteProjectQueryView::new(project_id);
    tx.execute(&view).await.map_err(|e| {
        log_db_error("projects/project_id/delete", &e);
        DeleteProjectError::DatabaseError
    })?;

    Ok(())
}

#[utoipa::path(
    delete,
    path = "",
    summary = "Delete a project",
    description = "Deletes a project for good, with its tasks and its members' memberships. \
                   Reserved to the managers of the project. To archive without destroying, \
                   prefer `PATCH /api/v1/projects/{project_id}/close`.\n\n\
                   Idempotent once the rights are checked: deleting a project already \
                   missing from the database also answers `204`.",
    responses(
        (
            status = 204,
            description = "Project deleted. Empty body.",
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
#[delete("/")]
pub async fn delete_project(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<ProjectPathParams>,
) -> Result<impl Responder, DeleteProjectError> {
    let (mut tx, _) = begin_write(
        &state,
        auth_user.id,
        params.project_id(),
        None,
        Requirement::ManageProject,
    )
    .await?;
    let project_id = params.project_id();
    trigger_delete_project(&mut tx, project_id).await?;
    commit(tx, "projects/project_id/delete").await?;
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
