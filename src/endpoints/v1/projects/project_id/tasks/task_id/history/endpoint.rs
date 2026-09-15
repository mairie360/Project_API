use actix_web::http::StatusCode;
use actix_web::{post, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::tasks::collaboration::view::{AppendTaskHistoryQueryView, TaskHistoryEntry};
use crate::endpoints::v1::projects::project_id::tasks::task_id::history::view::AppendTaskHistoryView;
use crate::endpoints::v1::projects::project_id::tasks::task_id::TaskPathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum AppendTaskHistoryError {
    BadRequest,
    NotFound,
    DatabaseError,
}

impl std::fmt::Display for AppendTaskHistoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppendTaskHistoryError::BadRequest => write!(f, "Bad request."),
            AppendTaskHistoryError::NotFound => write!(f, "Unknown task."),
            AppendTaskHistoryError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for AppendTaskHistoryError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppendTaskHistoryError::BadRequest => StatusCode::BAD_REQUEST,
            AppendTaskHistoryError::NotFound => StatusCode::NOT_FOUND,
            AppendTaskHistoryError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_append_task_history(
    state: web::Data<AppState>,
    params: &TaskPathParams,
    user_id: u64,
    view: AppendTaskHistoryView,
) -> Result<TaskHistoryEntry, AppendTaskHistoryError> {
    if view.action.trim().is_empty() || view.label.trim().is_empty() {
        return Err(AppendTaskHistoryError::BadRequest);
    }
    if view
        .changes
        .as_ref()
        .is_some_and(|changes| !changes.is_object())
    {
        return Err(AppendTaskHistoryError::BadRequest);
    }

    let entries: Vec<TaskHistoryEntry> = state
        .get_smart_db()
        .fetch_all(&AppendTaskHistoryQueryView::new(
            params.project_id(),
            params.task_id(),
            user_id,
            view.action.trim(),
            view.label.trim(),
            view.changes.as_ref(),
        ))
        .await
        .map_err(|_| AppendTaskHistoryError::DatabaseError)?;

    entries
        .into_iter()
        .next()
        .ok_or(AppendTaskHistoryError::NotFound)
}

#[utoipa::path(
    post,
    path = "history",
    params(
        TaskPathParams
    ),
    request_body = AppendTaskHistoryView,
    responses(
        (status = 201, description = "History entry added", body = TaskHistoryEntry),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Unknown task in this project"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Tasks",
)]
#[post("/history")]
pub async fn append_task_history(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<TaskPathParams>,
    view: web::Json<AppendTaskHistoryView>,
) -> Result<impl Responder, AppendTaskHistoryError> {
    let entry =
        trigger_append_task_history(state, &params, auth_user.id, view.into_inner()).await?;
    Ok(HttpResponse::Created().json(entry))
}
