use actix_web::http::StatusCode;
use actix_web::{post, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::users::add_user_to_project::view::AddUserToProjectQueryView;
use crate::endpoints::v1::projects::project_id::users::post::view::AddUserToProjectView;
use crate::endpoints::v1::projects::project_id::ProjectPathParams;

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum AddUserToProjectError {
    DatabaseError,
    BadRequest,
    UserNotFound,
}

impl std::fmt::Display for AddUserToProjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddUserToProjectError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            AddUserToProjectError::BadRequest => {
                write!(f, "Bad request.")
            }
            AddUserToProjectError::UserNotFound => {
                write!(f, "User not found.")
            }
        }
    }
}

impl ResponseError for AddUserToProjectError {
    fn status_code(&self) -> StatusCode {
        match self {
            AddUserToProjectError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            AddUserToProjectError::BadRequest => StatusCode::BAD_REQUEST,
            AddUserToProjectError::UserNotFound => StatusCode::NOT_FOUND,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_add_user_to_project(
    state: web::Data<AppState>,
    project_id: u64,
    view: AddUserToProjectView,
) -> Result<(), AddUserToProjectError> {
    let query_view = AddUserToProjectQueryView::new(project_id, view.user_id);
    state
        .get_smart_db()
        .execute(query_view)
        .await
        .map_err(|_| AddUserToProjectError::DatabaseError)?;

    Ok(())
}

#[utoipa::path(
    post,
    params(
        ProjectPathParams,
    ),
    path = "",
    responses(
        (status = 200, description = "User added to project successfully"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    request_body = AddUserToProjectView,
    security(
        ("jwt" = [])
    ),
    tag = "Users",
)]
#[post("/")]
pub async fn add_user_to_project(
    state: web::Data<AppState>,
    _: AuthenticatedUser,
    view: web::Json<AddUserToProjectView>,
    params: web::Path<ProjectPathParams>,
) -> Result<impl Responder, AddUserToProjectError> {
    let view = view
        .try_into()
        .map_err(|_| AddUserToProjectError::BadRequest)?;
    trigger_add_user_to_project(state, params.project_id, view).await?;
    Ok(HttpResponse::Ok().finish())
}
