use actix_web::http::StatusCode;
use actix_web::{delete, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::pool::AppState;
use mairie360_api_lib::security::AuthenticatedUser;

use crate::database::tasks::delete_task::query::delete_task_query;
use crate::database::tasks::delete_task::view::DeleteTaskQueryView;
use crate::endpoints::v1::projects::project_id::tasks::task_id::TaskPathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum DeleteTaskError {
    DatabaseError,
    UnknownTask,
}

impl std::fmt::Display for DeleteTaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeleteTaskError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            DeleteTaskError::UnknownTask => {
                write!(f, "Unknown task.")
            }
        }
    }
}

impl ResponseError for DeleteTaskError {
    fn status_code(&self) -> StatusCode {
        match self {
            DeleteTaskError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            DeleteTaskError::UnknownTask => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_delete_task(
    state: web::Data<AppState>,
    _project_id: u64,
    task_id: u64,
) -> Result<(), DeleteTaskError> {
    let pool = match state.db_pool.clone() {
        Some(pool) => pool,
        None => return Err(DeleteTaskError::DatabaseError),
    };

    let view = DeleteTaskQueryView::new(task_id);
    let result = delete_task_query(view, pool)
        .await
        .map_err(|_| DeleteTaskError::DatabaseError)?;

    // update cache

    if result == 0 {
        return Err(DeleteTaskError::UnknownTask);
    }
    Ok(())
}

#[utoipa::path(
    delete,
    path = "",
    responses(
        (status = 204, description = "Task deleted successfully"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    params(
        TaskPathParams
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Tasks",
)]
#[delete("/")]
pub async fn delete_task(
    state: web::Data<AppState>,
    _: AuthenticatedUser,
    params: web::Path<TaskPathParams>,
) -> Result<impl Responder, DeleteTaskError> {
    trigger_delete_task(state, params.project_id(), params.task_id).await?;
    Ok(HttpResponse::NoContent().finish())
}
