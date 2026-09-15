use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::tasks::collaboration::view::{
    GetTaskCollaborationQueryView, TaskCollaborationRow,
};
use crate::endpoints::v1::projects::project_id::tasks::task_id::collaboration::view::TaskCollaborationView;
use crate::endpoints::v1::projects::project_id::tasks::task_id::TaskPathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum GetTaskCollaborationError {
    NotFound,
    DatabaseError,
}

impl std::fmt::Display for GetTaskCollaborationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetTaskCollaborationError::NotFound => write!(f, "Unknown task."),
            GetTaskCollaborationError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for GetTaskCollaborationError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetTaskCollaborationError::NotFound => StatusCode::NOT_FOUND,
            GetTaskCollaborationError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_get_task_collaboration(
    state: web::Data<AppState>,
    project_id: u64,
    task_id: u64,
) -> Result<TaskCollaborationView, GetTaskCollaborationError> {
    let rows: Vec<TaskCollaborationRow> = state
        .get_smart_db()
        .fetch_all(&GetTaskCollaborationQueryView::new(project_id, task_id))
        .await
        .map_err(|_| GetTaskCollaborationError::DatabaseError)?;

    rows.into_iter()
        .next()
        .map(Into::into)
        .ok_or(GetTaskCollaborationError::NotFound)
}

#[utoipa::path(
    get,
    path = "collaboration",
    params(
        TaskPathParams
    ),
    responses(
        (status = 200, description = "Task comments and history", body = TaskCollaborationView),
        (status = 404, description = "Unknown task in this project"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Tasks",
)]
#[get("/collaboration")]
pub async fn get_task_collaboration(
    state: web::Data<AppState>,
    _: AuthenticatedUser,
    params: web::Path<TaskPathParams>,
) -> Result<impl Responder, GetTaskCollaborationError> {
    let result =
        trigger_get_task_collaboration(state, params.project_id(), params.task_id()).await?;
    Ok(HttpResponse::Ok().json(result))
}
