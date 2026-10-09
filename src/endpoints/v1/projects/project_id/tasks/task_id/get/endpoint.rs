use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::tasks::get_project_tasks::view::Task;
use crate::database::tasks::get_task::view::GetTaskQueryView;
use crate::endpoints::db_error::log_db_error;
use crate::endpoints::v1::projects::access::{require_access, AccessDenied, Requirement};
use crate::endpoints::v1::projects::project_id::tasks::get::view::TaskView;
use crate::endpoints::v1::projects::project_id::tasks::task_id::TaskPathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum GetTaskError {
    NotFound,
    DatabaseError,
}

impl std::fmt::Display for GetTaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetTaskError::NotFound => write!(f, "Unknown task."),
            GetTaskError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for GetTaskError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetTaskError::NotFound => StatusCode::NOT_FOUND,
            GetTaskError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

impl From<AccessDenied> for GetTaskError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            // Reading a task only needs the project to be visible: `Forbidden` cannot happen.
            AccessDenied::NotFound | AccessDenied::Forbidden => GetTaskError::NotFound,
            AccessDenied::DatabaseError => GetTaskError::DatabaseError,
        }
    }
}

async fn trigger_get_task(
    state: web::Data<AppState>,
    project_id: u64,
    task_id: u64,
) -> Result<TaskView, GetTaskError> {
    let rows: Vec<Task> = state
        .get_smart_db()
        .fetch_all(&GetTaskQueryView::new(project_id, task_id))
        .await
        .map_err(|e| {
            log_db_error("projects/project_id/tasks/task_id/get", &e);
            GetTaskError::DatabaseError
        })?;
    rows.into_iter()
        .next()
        .map(Into::into)
        .ok_or(GetTaskError::NotFound)
}

#[utoipa::path(
    get,
    path = "",
    summary = "Read one task of a project",
    description = "Returns one task of a project visible to the caller, with the same fields as \
                   the items of `GET /api/v1/projects/{project_id}/tasks/`. Lets a client check a \
                   task (its assignee, its status) without reading every task of the project \
                   (MAIR-474). The rights on the task are not checked here: any caller who sees the \
                   project sees its tasks, as in the task list.",
    params(
        TaskPathParams
    ),
    responses(
        (
            status = 200,
            description = "The task.",
            body = TaskView,
            example = json!({
                "id": 87,
                "title": "Consulter les riverains",
                "description": "Réunion publique à organiser avant le 15 octobre",
                "status": "InProgress",
                "priority": "High",
                "due_date": "2026-10-15T00:00:00Z",
                "archived_at": null,
                "assigned_to": 42,
                "fields": []
            })
        ),
        (
            status = 400,
            description = "A URL segment is not an integer.",
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
            status = 404,
            description = "Unknown project or task, task of another project, or project not visible \
                           to the caller.",
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
    security(
        ("jwt" = [])
    ),
    tag = "Tasks",
)]
#[get("/")]
pub async fn get_task(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<TaskPathParams>,
) -> Result<impl Responder, GetTaskError> {
    require_access(
        &state,
        auth_user.id,
        params.project_id(),
        Some(params.task_id()),
        Requirement::ViewProject,
    )
    .await?;
    let task = trigger_get_task(state, params.project_id(), params.task_id()).await?;
    Ok(HttpResponse::Ok().json(task))
}
