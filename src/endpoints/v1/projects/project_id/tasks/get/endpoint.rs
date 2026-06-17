use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::pool::AppState;
use mairie360_api_lib::security::AuthenticatedUser;

use crate::endpoints::v1::projects::project_id::tasks::get::view::GetTasksResultView;
use crate::endpoints::v1::projects::project_id::ProjectPathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum GetTasksError {
    BadRequest,
    DatabaseError,
}

impl std::fmt::Display for GetTasksError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetTasksError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            GetTasksError::BadRequest => {
                write!(f, "Bad request.")
            }
        }
    }
}

impl ResponseError for GetTasksError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetTasksError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            GetTasksError::BadRequest => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_get_project_tasks(
    state: web::Data<AppState>,
    project_id: u64,
) -> Result<GetTasksResultView, GetTasksError> {
    // get cache

    let pool = match state.db_pool.clone() {
        Some(pool) => pool,
        None => return Err(GetTasksError::DatabaseError),
    };

    //query

    // update cache

    Ok(GetTasksResultView { tasks: vec![] })
}

#[utoipa::path(
    get,
    path = "",
    responses(
        (status = 200, description = "Tasks retrieved successfully", body = GetTasksResultView),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Tasks",
)]
#[get("/")]
pub async fn get_project_tasks(
    state: web::Data<AppState>,
    _: AuthenticatedUser,
    params: web::Path<ProjectPathParams>,
) -> Result<impl Responder, GetTasksError> {
    let result = trigger_get_project_tasks(state, params.project_id).await?;
    Ok(HttpResponse::Ok().json(result))
}
