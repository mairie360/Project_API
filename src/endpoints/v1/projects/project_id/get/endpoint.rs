use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::pool::AppState;
use mairie360_api_lib::security::AuthenticatedUser;

use crate::endpoints::v1::projects::project_id::get::view::GetProjectResultView;
use crate::endpoints::v1::projects::project_id::ProjectPathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum GetProjectError {
    BadRequest,
    DatabaseError,
}

impl std::fmt::Display for GetProjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetProjectError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            GetProjectError::BadRequest => {
                write!(f, "Bad request.")
            }
        }
    }
}

impl ResponseError for GetProjectError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetProjectError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            GetProjectError::BadRequest => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_get_project(
    state: web::Data<AppState>,
    project_id: u64,
) -> Result<(), GetProjectError> {
    // ) -> Result<GetProjectResultView, GetProjectError> {
    // get cache

    let pool = match state.db_pool.clone() {
        Some(pool) => pool,
        None => return Err(GetProjectError::DatabaseError),
    };

    //query

    // update cache

    Ok(())
}

#[utoipa::path(
    get,
    path = "",
    responses(
        (status = 200, description = "Project retrieved successfully", body = GetProjectResultView),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Projects",
)]
#[get("/")]
pub async fn get_project(
    state: web::Data<AppState>,
    _: AuthenticatedUser,
    params: web::Path<ProjectPathParams>,
) -> Result<impl Responder, GetProjectError> {
    let result = trigger_get_project(state, params.project_id).await?;
    Ok(HttpResponse::Ok().json(result))
}
