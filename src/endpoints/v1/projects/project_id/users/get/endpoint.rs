use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::pool::AppState;
use mairie360_api_lib::security::AuthenticatedUser;

use crate::database::users::get_project_users::query::get_project_users_query;
use crate::database::users::get_project_users::view::GetProjectUsersQueryView;
use crate::endpoints::v1::projects::project_id::users::get::view::{
    GetProjectUsersResultView, User,
};
use crate::endpoints::v1::projects::project_id::ProjectPathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum GetProjectUsersError {
    BadRequest,
    DatabaseError,
}

impl std::fmt::Display for GetProjectUsersError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetProjectUsersError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            GetProjectUsersError::BadRequest => {
                write!(f, "Bad request.")
            }
        }
    }
}

impl ResponseError for GetProjectUsersError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetProjectUsersError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            GetProjectUsersError::BadRequest => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_get_project_users(
    state: web::Data<AppState>,
    project_id: u64,
) -> Result<GetProjectUsersResultView, GetProjectUsersError> {
    // get cache

    let pool = match state.db_pool.clone() {
        Some(pool) => pool,
        None => return Err(GetProjectUsersError::DatabaseError),
    };

    let view = GetProjectUsersQueryView::new(project_id);
    let result = get_project_users_query(view, pool)
        .await
        .map_err(|_| GetProjectUsersError::DatabaseError)?;

    // update cache

    Ok(GetProjectUsersResultView {
        users: result
            .into_iter()
            .map(|user| User { id: user as u64 })
            .collect(),
    })
}

#[utoipa::path(
    get,
    params(
        ProjectPathParams,
    ),
    path = "",
    responses(
        (status = 200, description = "Project users retrieved successfully", body = GetProjectUsersResultView),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Users",
)]
#[get("/")]
pub async fn get_project_users(
    state: web::Data<AppState>,
    _: AuthenticatedUser,
    params: web::Path<ProjectPathParams>,
) -> Result<impl Responder, GetProjectUsersError> {
    let result = trigger_get_project_users(state, params.project_id).await?;
    Ok(HttpResponse::Ok().json(result))
}
