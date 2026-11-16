use actix_web::http::StatusCode;
use actix_web::{delete, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::smart_db::SmartTransaction;
use mairie360_api_lib::state::AppState;

use crate::endpoints::db_error::log_db_error;

use crate::database::tasks::delete_task::view::DeleteTaskQueryView;
use crate::endpoints::v1::projects::access::{begin_write, commit, AccessDenied, Requirement};
use crate::endpoints::v1::projects::project_id::tasks::task_id::TaskPathParams;

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum DeleteTaskError {
    Forbidden,
    NotFound,
    DatabaseError,
    UnknownTask,
}

impl std::fmt::Display for DeleteTaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeleteTaskError::Forbidden => write!(f, "Forbidden."),
            DeleteTaskError::NotFound => write!(f, "Not found."),
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
            DeleteTaskError::Forbidden => StatusCode::FORBIDDEN,
            DeleteTaskError::NotFound => StatusCode::NOT_FOUND,
            DeleteTaskError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            DeleteTaskError::UnknownTask => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_delete_task(
    tx: &mut SmartTransaction,
    _project_id: u64,
    task_id: u64,
) -> Result<(), DeleteTaskError> {
    let view = DeleteTaskQueryView::new(task_id);
    tx.execute(&view).await.map_err(|e| {
        log_db_error("projects/project_id/tasks/task_id/delete", &e);
        DeleteTaskError::DatabaseError
    })?;

    Ok(())
}

#[utoipa::path(
    delete,
    path = "",
    summary = "Delete a task",
    description = "Deletes a task for good, with its comments and its history. \
               Reserved to the managers of the project: an agent assigned to the task cannot \
               delete it, only change its status.\n\n\
               Idempotent once the rights are checked: deleting a task already \
               missing from the database also answers `204`.",
    responses(
        (
            status = 204,
            description = "Task deleted. Empty body.",
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
            description = "Project or task that does not exist, or project the caller cannot see. A task belonging to another project is treated as missing.",
            body = String,
            content_type = "text/plain",
            example = json!("Unknown task.")
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
    auth_user: AuthenticatedUser,
    params: web::Path<TaskPathParams>,
) -> Result<impl Responder, DeleteTaskError> {
    let (mut tx, _) = begin_write(
        &state,
        auth_user.id,
        params.project_id(),
        Some(params.task_id()),
        Requirement::ManageProject,
    )
    .await?;
    trigger_delete_task(&mut tx, params.project_id(), params.task_id).await?;
    commit(tx, "projects/project_id/tasks/task_id/delete").await?;
    Ok(HttpResponse::NoContent().finish())
}

impl From<AccessDenied> for DeleteTaskError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => DeleteTaskError::NotFound,
            AccessDenied::Forbidden => DeleteTaskError::Forbidden,
            AccessDenied::DatabaseError => DeleteTaskError::DatabaseError,
        }
    }
}
