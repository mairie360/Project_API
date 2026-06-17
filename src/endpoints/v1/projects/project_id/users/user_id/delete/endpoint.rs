use actix_web::http::StatusCode;
use actix_web::{delete, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::pool::AppState;
use mairie360_api_lib::security::AuthenticatedUser;

use crate::endpoints::v1::projects::project_id::users::user_id::ProjectUserPathParams;
use crate::endpoints::v1::projects::project_id::ProjectPathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum RemoveUserFromProjectError {
    DatabaseError,
    UnknownUser,
}

impl std::fmt::Display for RemoveUserFromProjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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
    let pool = match state.db_pool.clone() {
        Some(pool) => pool,
        None => return Err(RemoveUserFromProjectError::DatabaseError),
    };

    //query

    // update cache

    Ok(())
}

#[utoipa::path(
    delete,
    params(
        ProjectUserPathParams,
    ),
    responses(
        (status = 204, description = "User removed from project successfully"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Users",
)]
#[delete("/")]
pub async fn remove_user_from_project(
    state: web::Data<AppState>,
    _: AuthenticatedUser,
    params: web::Path<ProjectUserPathParams>,
) -> Result<impl Responder, RemoveUserFromProjectError> {
    let project_id = params.project_id();
    let user_id = params.user_id();
    trigger_remove_user_from_project(state, project_id, user_id).await?;
    Ok(HttpResponse::NoContent().finish())
}
