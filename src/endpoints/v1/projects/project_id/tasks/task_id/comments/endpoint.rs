use actix_web::http::StatusCode;
use actix_web::{post, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::endpoints::db_error::log_db_error;

use crate::database::tasks::collaboration::view::{AddTaskCommentQueryView, TaskComment};
use crate::endpoints::v1::projects::access::{require_access, AccessDenied, Requirement};
use crate::endpoints::v1::projects::project_id::tasks::task_id::comments::view::{
    AddTaskCommentView, MAX_COMMENT_LENGTH,
};
use crate::endpoints::v1::projects::project_id::tasks::task_id::TaskPathParams;
use crate::endpoints::validation::ValidatedJson;

#[derive(Debug, Clone, PartialEq)]
pub enum AddTaskCommentError {
    Forbidden,
    BadRequest,
    NotFound,
    DatabaseError,
}

impl std::fmt::Display for AddTaskCommentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddTaskCommentError::Forbidden => write!(f, "Forbidden."),
            AddTaskCommentError::BadRequest => write!(f, "Bad request."),
            AddTaskCommentError::NotFound => write!(f, "Unknown task."),
            AddTaskCommentError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for AddTaskCommentError {
    fn status_code(&self) -> StatusCode {
        match self {
            AddTaskCommentError::Forbidden => StatusCode::FORBIDDEN,
            AddTaskCommentError::BadRequest => StatusCode::BAD_REQUEST,
            AddTaskCommentError::NotFound => StatusCode::NOT_FOUND,
            AddTaskCommentError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_add_task_comment(
    state: web::Data<AppState>,
    params: &TaskPathParams,
    user_id: u64,
    view: AddTaskCommentView,
) -> Result<TaskComment, AddTaskCommentError> {
    let message = view.message.trim();
    if message.is_empty() || message.chars().count() > MAX_COMMENT_LENGTH {
        return Err(AddTaskCommentError::BadRequest);
    }

    let comments: Vec<TaskComment> = state
        .get_smart_db()
        .fetch_all(&AddTaskCommentQueryView::new(
            params.project_id(),
            params.task_id(),
            user_id,
            message,
        ))
        .await
        .map_err(|e| {
            log_db_error("projects/project_id/tasks/task_id/comments", &e);
            AddTaskCommentError::DatabaseError
        })?;

    comments
        .into_iter()
        .next()
        .ok_or(AddTaskCommentError::NotFound)
}

#[utoipa::path(
    post,
    path = "comments",
    summary = "Comment on a task",
    description = "Adds a comment to the discussion thread of a task. Open to the project managers \
                   and to the agent assigned to the task.\n\n\
                   The message is trimmed, then must hold 1 to 2000 characters: an empty or too \
                   long message is refused with `400`.\n\n\
                   The author comes from the JWT, never from the body. The created comment is \
                   returned with its id and date, and then appears in \
                   `GET …/tasks/{task_id}/collaboration`.",
    params(
        TaskPathParams
    ),
    request_body(
        content = AddTaskCommentView,
        description = "Text of the comment.",
        example = json!({ "message": "La réunion publique est calée au 3 octobre." })
    ),
    responses(
        (
            status = 201,
            description = "Comment added.",
            body = TaskComment,
            example = json!({
                "id": "comment-31",
                "message": "La réunion publique est calée au 3 octobre.",
                "author": { "id": "user-42", "name": "Jean Dupont" },
                "createdAt": "2026-09-14T09:12:00.000Z"
            })
        ),
        (
            status = 400,
            description = "Malformed JSON body, URL segment not an integer, or `message` empty, longer than 2000 characters, containing `<` / `>` or a control character other than line breaks and tabs.",
            body = String,
            content_type = "text/plain",
            example = json!("Invalid `message`: must not contain `<` or `>`")
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
#[post("/comments")]
pub async fn add_task_comment(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<TaskPathParams>,
    view: ValidatedJson<AddTaskCommentView>,
) -> Result<impl Responder, AddTaskCommentError> {
    require_access(
        &state,
        auth_user.id,
        params.project_id(),
        Some(params.task_id()),
        Requirement::ActOnTask,
    )
    .await?;
    let comment = trigger_add_task_comment(state, &params, auth_user.id, view.into_inner()).await?;
    Ok(HttpResponse::Created().json(comment))
}

impl From<AccessDenied> for AddTaskCommentError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => AddTaskCommentError::NotFound,
            AccessDenied::Forbidden => AddTaskCommentError::Forbidden,
            AccessDenied::DatabaseError => AddTaskCommentError::DatabaseError,
        }
    }
}
