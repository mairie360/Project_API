use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::pool::AppState;
use mairie360_api_lib::security::AuthenticatedUser;

use crate::database::project::get_projects::query::get_projects_query;
use crate::database::project::get_projects::view::GetProjectsQueryView;
use crate::endpoints::v1::projects::get::view::GetProjectsResultView;

#[derive(Debug, Clone, PartialEq)]
pub enum GetProjectsError {
    BadRequest,
    DatabaseError,
}

impl std::fmt::Display for GetProjectsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetProjectsError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            GetProjectsError::BadRequest => {
                write!(f, "Bad request.")
            }
        }
    }
}

impl ResponseError for GetProjectsError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetProjectsError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            GetProjectsError::BadRequest => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_get_projects(
    state: web::Data<AppState>,
    user_id: u64,
) -> Result<GetProjectsResultView, GetProjectsError> {
    // get cache

    let pool = match state.db_pool.clone() {
        Some(pool) => pool,
        None => return Err(GetProjectsError::DatabaseError),
    };

    //query
    let view = GetProjectsQueryView::new(user_id);
    let result = get_projects_query(view, pool)
        .await
        .map_err(|_| GetProjectsError::DatabaseError)?;

    // update cache

    Ok(GetProjectsResultView {
        projects: result.into_iter().map(|p| p.into()).collect(),
    })
}

#[utoipa::path(
    get,
    path = "",
    responses(
        (status = 200, description = "Projects retrieved successfully", body = GetProjectsResultView),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Projects",
)]
#[get("/")]
pub async fn get_projects(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl Responder, GetProjectsError> {
    let result = trigger_get_projects(state, auth_user.id).await?;
    Ok(HttpResponse::Ok().json(result))
}
