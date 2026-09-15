use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::tasks::get_project_tasks::view::{GetProjectTasksQueryView, Task};
use crate::endpoints::v1::projects::access::{require_access, AccessDenied, Requirement};
use crate::endpoints::v1::projects::project_id::tasks::get::view::GetTasksResultView;
use crate::endpoints::v1::projects::project_id::ProjectPathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum GetTasksError {
    Forbidden,
    NotFound,
    BadRequest,
    DatabaseError,
}

impl std::fmt::Display for GetTasksError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetTasksError::Forbidden => write!(f, "Forbidden."),
            GetTasksError::NotFound => write!(f, "Not found."),
            GetTasksError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            GetTasksError::BadRequest => {
                write!(f, "Bad request.")
            }
        }
    }
}

impl ResponseError for GetTasksError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetTasksError::Forbidden => StatusCode::FORBIDDEN,
            GetTasksError::NotFound => StatusCode::NOT_FOUND,
            GetTasksError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            GetTasksError::BadRequest => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_get_project_tasks(
    state: web::Data<AppState>,
    project_id: u64,
) -> Result<GetTasksResultView, GetTasksError> {
    let view = GetProjectTasksQueryView::new(project_id);
    let result: Vec<Task> = state
        .get_smart_db()
        .fetch_all(&view)
        .await
        .map_err(|_| GetTasksError::DatabaseError)?;

    Ok(GetTasksResultView {
        tasks: result.into_iter().map(|t| t.into()).collect(),
    })
}

#[utoipa::path(
    get,
    params(
        ProjectPathParams,
    ),
    path = "",
    responses(
        (status = 200, description = "Tasks retrieved successfully", body = GetTasksResultView),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Tasks",
)]
#[get("/")]
pub async fn get_project_tasks(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<ProjectPathParams>,
) -> Result<impl Responder, GetTasksError> {
    require_access(
        &state,
        auth_user.id,
        params.project_id(),
        None,
        Requirement::ViewProject,
    )
    .await?;
    let result = trigger_get_project_tasks(state, params.project_id).await?;
    Ok(HttpResponse::Ok().json(result))
}

impl From<AccessDenied> for GetTasksError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => GetTasksError::NotFound,
            AccessDenied::Forbidden => GetTasksError::Forbidden,
            AccessDenied::DatabaseError => GetTasksError::DatabaseError,
        }
    }
}
