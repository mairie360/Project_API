use actix_web::http::StatusCode;
use actix_web::{patch, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::tasks::create_task::view::{
    TaskPriority as DbTaskPriority, TaskStatus as DbTaskStatus,
};
use crate::database::tasks::patch_task::view::PatchTaskQueryView;
use crate::endpoints::v1::projects::project_id::get::view::TaskPriority;
use crate::endpoints::v1::projects::project_id::tasks::task_id::patch::view::PatchTaskView;
use crate::endpoints::v1::projects::project_id::tasks::task_id::TaskPathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum PatchTaskError {
    BadRequest,
    NotFound,
    DatabaseError,
}

impl std::fmt::Display for PatchTaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PatchTaskError::BadRequest => write!(f, "Bad request."),
            PatchTaskError::NotFound => write!(f, "Unknown task."),
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
            PatchTaskError::NotFound => StatusCode::NOT_FOUND,
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
    if view
        .name
        .as_deref()
        .is_some_and(|name| name.trim().is_empty())
    {
        return Err(PatchTaskError::BadRequest);
    }
    let status = view
        .status
        .map(|status| DbTaskStatus::from(status.to_string()));
    // La base ne connaît pas « urgent » : la priorité la plus haute est retenue.
    let priority = view.priority.map(|priority| match priority {
        TaskPriority::Urgent => DbTaskPriority::High,
        priority => DbTaskPriority::from(priority.to_string()),
    });
    if status == Some(DbTaskStatus::Error) || priority == Some(DbTaskPriority::Error) {
        return Err(PatchTaskError::BadRequest);
    }

    let updated: bool = state
        .get_smart_db()
        .fetch_scalar(&PatchTaskQueryView::new(
            project_id,
            task_id,
            view.name.as_deref().map(str::trim),
            status,
            priority,
            view.due_date,
            view.assigned_to,
        ))
        .await
        .map_err(|_| PatchTaskError::DatabaseError)?;

    if updated {
        Ok(())
    } else {
        Err(PatchTaskError::NotFound)
    }
}

#[utoipa::path(
    patch,
    path = "",
    responses(
        (status = 204, description = "Task patched successfully"),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Unknown task in this project"),
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
    trigger_patch_task(
        state,
        params.project_id(),
        params.task_id(),
        view.into_inner(),
    )
    .await?;
    Ok(HttpResponse::NoContent().finish())
}
