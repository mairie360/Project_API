use actix_web::http::StatusCode;
use actix_web::{post, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::tasks::collaboration::view::{AddTaskCommentQueryView, TaskComment};
use crate::endpoints::v1::projects::access::{require_access, AccessDenied, Requirement};
use crate::endpoints::v1::projects::project_id::tasks::task_id::comments::view::{
    AddTaskCommentView, MAX_COMMENT_LENGTH,
};
use crate::endpoints::v1::projects::project_id::tasks::task_id::TaskPathParams;

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
        .map_err(|_| AddTaskCommentError::DatabaseError)?;

    comments
        .into_iter()
        .next()
        .ok_or(AddTaskCommentError::NotFound)
}

#[utoipa::path(
    post,
    path = "comments",
    params(
        TaskPathParams
    ),
    request_body = AddTaskCommentView,
    responses(
        (status = 201, description = "Comment added", body = TaskComment),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Unknown task in this project"),
        (status = 500, description = "Internal server error")
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
    view: web::Json<AddTaskCommentView>,
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
