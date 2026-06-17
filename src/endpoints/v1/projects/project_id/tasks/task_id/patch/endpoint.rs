use actix_web::http::StatusCode;
use actix_web::{patch, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::pool::AppState;
use mairie360_api_lib::security::AuthenticatedUser;

use crate::endpoints::v1::projects::project_id;
use crate::endpoints::v1::projects::project_id::tasks::task_id::patch::view::PatchTaskView;
use crate::endpoints::v1::projects::project_id::tasks::task_id::TaskPathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum PatchTaskError {
    BadRequest,
    DatabaseError,
}

impl std::fmt::Display for PatchTaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PatchTaskError::BadRequest => {
                write!(f, "Bad request.")
            }
            PatchTaskError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for PatchTaskError {
    fn status_code(&self) -> StatusCode {
        match self {
            PatchTaskError::BadRequest => StatusCode::BAD_REQUEST,
            PatchTaskError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_patch_task(
    state: web::Data<AppState>,
    project_id: u64,
    task_id: u64,
    view: PatchTaskView,
) -> Result<(), PatchTaskError> {
    let pool = match state.db_pool.clone() {
        Some(pool) => pool,
        None => return Err(PatchTaskError::DatabaseError),
    };

    //query

    // update cache

    Ok(())
}

#[utoipa::path(
    patch,
    path = "",
    responses(
        (status = 200, description = "Task patched successfully"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    params(
        TaskPathParams
    ),
    request_body = PatchTaskView,
    security(
        ("jwt" = [])
    ),
    tag = "Tasks",
)]
#[patch("/")]
pub async fn patch_task(
    state: web::Data<AppState>,
    _: AuthenticatedUser,
    params: web::Path<TaskPathParams>,
    view: web::Json<PatchTaskView>,
) -> Result<impl Responder, PatchTaskError> {
    let view = view.try_into().map_err(|_| PatchTaskError::BadRequest)?;
    trigger_patch_task(state, params.project_id(), params.task_id(), view).await?;
    Ok(HttpResponse::NoContent().finish())
}
