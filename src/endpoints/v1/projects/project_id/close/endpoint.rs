use actix_web::http::StatusCode;
use actix_web::{patch, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::pool::AppState;
use mairie360_api_lib::security::AuthenticatedUser;

use crate::endpoints::v1::projects::project_id::ProjectPathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum PatchMessageError {
    BadRequest,
    DatabaseError,
    UnknownProject,
}

impl std::fmt::Display for PatchMessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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
    let pool = match state.db_pool.clone() {
        Some(pool) => pool,
        None => return Err(PatchMessageError::DatabaseError),
    };

    //query

    // update cache

    Ok(())
}

#[utoipa::path(
    patch,
    path = "",
    responses(
        (status = 200, description = "Project closed successfully"),
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
#[patch("/close")]
pub async fn close_project(
    state: web::Data<AppState>,
    _: AuthenticatedUser,
    params: web::Path<ProjectPathParams>,
) -> Result<impl Responder, PatchMessageError> {
    let project_id = params.project_id();
    trigger_close_project(state, project_id).await?;
    Ok(HttpResponse::Ok())
}
