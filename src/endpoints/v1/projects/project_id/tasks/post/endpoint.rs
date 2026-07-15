use actix_web::http::StatusCode;
use actix_web::{post, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::pool::AppState;
use mairie360_api_lib::security::AuthenticatedUser;

use crate::database::tasks::create_task::query::create_task_query;
use crate::database::tasks::create_task::view::CreateTaskQueryView;
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
    let pool = match state.db_pool.clone() {
        Some(pool) => pool,
        None => return Err(CreateTaskError::DatabaseError),
    };

    let name = view.name();

    let view = CreateTaskQueryView::new(
        project_id,
        name,
        view.status().unwrap().to_string().into(),
        view.priority().unwrap().to_string().into(),
        *view.due_date(),
        *view.assigned_to(),
    );
    let result = create_task_query(view, pool)
        .await
        .map_err(|_| CreateTaskError::DatabaseError)?;

    // update cache

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
