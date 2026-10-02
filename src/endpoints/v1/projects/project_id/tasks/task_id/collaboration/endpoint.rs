use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::endpoints::db_error::log_db_error;

use crate::database::tasks::collaboration::view::{
    GetTaskCollaborationQueryView, TaskCollaborationRow,
};
use crate::endpoints::pagination::{Page, PageParams};
use crate::endpoints::v1::projects::access::{require_access, AccessDenied, Requirement};
use crate::endpoints::v1::projects::project_id::tasks::task_id::collaboration::view::TaskCollaborationView;
use crate::endpoints::v1::projects::project_id::tasks::task_id::TaskPathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum GetTaskCollaborationError {
    Forbidden,
    NotFound,
    DatabaseError,
}

impl std::fmt::Display for GetTaskCollaborationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetTaskCollaborationError::Forbidden => write!(f, "Forbidden."),
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
            GetTaskCollaborationError::Forbidden => StatusCode::FORBIDDEN,
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
    page: Page,
) -> Result<TaskCollaborationView, GetTaskCollaborationError> {
    let rows: Vec<TaskCollaborationRow> = state
        .get_smart_db()
        .fetch_all(&GetTaskCollaborationQueryView::new(
            project_id,
            task_id,
            page.limit,
            page.offset,
        ))
        .await
        .map_err(|e| {
            log_db_error("projects/project_id/tasks/task_id/collaboration", &e);
            GetTaskCollaborationError::DatabaseError
        })?;

    rows.into_iter()
        .next()
        .map(Into::into)
        .ok_or(GetTaskCollaborationError::NotFound)
}

#[utoipa::path(
    get,
    path = "collaboration",
    summary = "Read the comments and history of a task",
    description = "Returns, in one request, one page of the discussion thread and one page of the \
                   activity log of a task. Open to the project managers and to the agent assigned \
                   to the task.\n\n\
                   The two lists are sorted in opposite orders: `comments` from oldest to newest \
                   (reading order of a discussion), `history` from newest to oldest (order of a \
                   log). `limit` / `offset` page each list independently in its own order; \
                   `comments_total` and `history_total` give the full counts.\n\n\
                   The history is written by the server only: `task_created` when the task is \
                   created, `task_updated` for every `PATCH` changing other fields than the \
                   status, and `status_changed` for every status change, each signed by the \
                   caller of the request that made it. There is no route to add an entry.",
    params(
        TaskPathParams,
        PageParams
    ),
    responses(
        (
            status = 200,
            description = "One page of the comments and one page of the history of the task.",
            body = TaskCollaborationView,
            example = json!({
                "comments": [
                    {
                        "id": "comment-31",
                        "message": "La réunion publique est calée au 3 octobre.",
                        "author": { "id": "user-42", "name": "Jean Dupont" },
                        "createdAt": "2026-09-14T09:12:00.000Z"
                    }
                ],
                "comments_total": 1,
                "history": [
                    {
                        "id": "history-8",
                        "action": "status_changed",
                        "label": "Status changed: todo → in_progress",
                        "author": { "id": "user-42", "name": "Jean Dupont" },
                        "createdAt": "2026-09-15T10:04:00.000Z",
                        "changes": { "status": { "from": "todo", "to": "in_progress" } }
                    },
                    {
                        "id": "history-5",
                        "action": "task_created",
                        "label": "Task created",
                        "author": { "id": "user-51", "name": "Amina Bensaïd" },
                        "createdAt": "2026-09-12T08:30:00.000Z"
                    }
                ],
                "history_total": 2
            })
        ),
        (
            status = 400,
            description = "A URL segment is not an integer, or `limit` / `offset` is not a non-negative integer.",
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
            description = "Neither a manager of the project nor the agent assigned to the task.",
            body = String,
            content_type = "text/plain",
            example = json!("Forbidden.")
        ),
        (
            status = 404,
            description = "Unknown project or task, or project not visible to the caller.",
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
#[get("/collaboration")]
pub async fn get_task_collaboration(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<TaskPathParams>,
    page: web::Query<PageParams>,
) -> Result<impl Responder, GetTaskCollaborationError> {
    require_access(
        &state,
        auth_user.id,
        params.project_id(),
        Some(params.task_id()),
        Requirement::ActOnTask,
    )
    .await?;
    let result =
        trigger_get_task_collaboration(state, params.project_id(), params.task_id(), page.page())
            .await?;
    Ok(HttpResponse::Ok().json(result))
}

impl From<AccessDenied> for GetTaskCollaborationError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => GetTaskCollaborationError::NotFound,
            AccessDenied::Forbidden => GetTaskCollaborationError::Forbidden,
            AccessDenied::DatabaseError => GetTaskCollaborationError::DatabaseError,
        }
    }
}
