use actix_web::http::StatusCode;
use actix_web::{delete, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::pool::AppState;
use mairie360_api_lib::security::AuthenticatedUser;

use crate::endpoints::v1::projects::project_id::ProjectPathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum DeleteProjectError {
    DatabaseError,
    UnknownEvent,
}

impl std::fmt::Display for DeleteProjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeleteProjectError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            DeleteProjectError::UnknownEvent => {
                write!(f, "Unknown event.")
            }
        }
    }
}

impl ResponseError for DeleteProjectError {
    fn status_code(&self) -> StatusCode {
        match self {
            DeleteProjectError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            DeleteProjectError::UnknownEvent => StatusCode::BAD_REQUEST,
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
    let pool = match state.db_pool.clone() {
        Some(pool) => pool,
        None => return Err(DeleteProjectError::DatabaseError),
    };

    //query

    // update cache

    Ok(())
}

#[utoipa::path(
    delete,
    path = "",
    responses(
        (status = 204, description = "Project deleted successfully"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
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
    _: AuthenticatedUser,
    params: web::Path<ProjectPathParams>,
) -> Result<impl Responder, DeleteProjectError> {
    let project_id = params.project_id();
    trigger_delete_project(state, project_id).await?;
    Ok(HttpResponse::NoContent().finish())
}
