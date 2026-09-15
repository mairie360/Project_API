use actix_web::http::StatusCode;
use actix_web::{post, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::tasks::create_task::view::{
    CreateTaskQueryView, TaskPriority as DbTaskPriority, TaskStatus,
};
use crate::endpoints::v1::projects::project_id::get::view::TaskPriority as ApiTaskPriority;
use crate::endpoints::v1::projects::project_id::tasks::post::view::{
    CreateTaskResultView, CreateTaskView,
};
use crate::endpoints::v1::projects::project_id::ProjectPathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum CreateTaskError {
    DatabaseError,
    BadRequest,
}

impl std::fmt::Display for CreateTaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CreateTaskError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            CreateTaskError::BadRequest => {
                write!(f, "Bad request.")
            }
        }
    }
}

impl ResponseError for CreateTaskError {
    fn status_code(&self) -> StatusCode {
        match self {
            CreateTaskError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            CreateTaskError::BadRequest => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_create_task(
    state: web::Data<AppState>,
    _user_id: u64,
    project_id: u64,
    view: CreateTaskView,
) -> Result<CreateTaskResultView, CreateTaskError> {
    let name = view.name().to_string();

    // Statut et priorité facultatifs : valeurs par défaut de la table (todo, medium).
    let status = view
        .status()
        .map(|status| status.to_string().into())
        .unwrap_or(TaskStatus::Todo);
    let priority = match view.priority() {
        Some(ApiTaskPriority::Urgent) => DbTaskPriority::High,
        Some(priority) => priority.to_string().into(),
        None => DbTaskPriority::Medium,
    };
    if status == TaskStatus::Error || priority == DbTaskPriority::Error {
        return Err(CreateTaskError::BadRequest);
    }
    let query_view = CreateTaskQueryView::new(
        project_id,
        &name,
        status,
        priority,
        *view.due_date(),
        *view.assigned_to(),
        view.fields(),
    );
    let result: i32 = state
        .get_smart_db()
        .fetch_scalar::<i32, _>(&query_view)
        .await
        .map_err(|_| CreateTaskError::DatabaseError)?;

    Ok(CreateTaskResultView {
        task_id: result as u64,
        name: name.to_string(),
        description: None,
    })
}

#[utoipa::path(
    post,
    params(
        ProjectPathParams,
    ),
    path = "",
    responses(
        (status = 200, description = "Project created successfully", body = CreateTaskResultView),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    request_body = CreateTaskView,
    security(
        ("jwt" = [])
    ),
    tag = "Tasks",
)]
#[post("/")]
pub async fn create_task(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    view: web::Json<CreateTaskView>,
    params: web::Path<ProjectPathParams>,
) -> Result<impl Responder, CreateTaskError> {
    let view = view.try_into().map_err(|_| CreateTaskError::BadRequest)?;
    let result = trigger_create_task(state, auth_user.id, params.project_id, view).await?;
    Ok(HttpResponse::Ok().json(result))
}
