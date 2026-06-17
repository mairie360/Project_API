use actix_web::http::StatusCode;
use actix_web::{post, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::pool::AppState;
use mairie360_api_lib::security::AuthenticatedUser;

use crate::endpoints::v1::projects::post::view::{CreateProjectResultView, CreateProjectView};

#[derive(Debug, Clone, PartialEq)]
pub enum CreateProjectError {
    DatabaseError,
    BadRequest,
}

impl std::fmt::Display for CreateProjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CreateProjectError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            CreateProjectError::BadRequest => {
                write!(f, "Bad request.")
            }
        }
    }
}

impl ResponseError for CreateProjectError {
    fn status_code(&self) -> StatusCode {
        match self {
            CreateProjectError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            CreateProjectError::BadRequest => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_create_project(
    state: web::Data<AppState>,
    user_id: u64,
    view: CreateProjectView,
) -> Result<(), CreateProjectError> {
    let pool = match state.db_pool.clone() {
        Some(pool) => pool,
        None => return Err(CreateProjectError::DatabaseError),
    };

    //query

    // update cache

    Ok(())
}

#[utoipa::path(
    post,
    path = "",
    responses(
        (status = 200, description = "Project created successfully", body = CreateProjectResultView),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    request_body = CreateProjectView,
    security(
        ("jwt" = [])
    ),
    tag = "Projects",
)]
#[post("/")]
pub async fn create_project(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    view: web::Json<CreateProjectView>,
) -> Result<impl Responder, CreateProjectError> {
    let view = view
        .try_into()
        .map_err(|_| CreateProjectError::BadRequest)?;
    let result = trigger_create_project(state, auth_user.id, view).await?;
    Ok(HttpResponse::Ok().json(result))
}
