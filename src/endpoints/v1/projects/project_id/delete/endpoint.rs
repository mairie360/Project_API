use actix_web::http::StatusCode;
use actix_web::{delete, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::pool::AppState;
use mairie360_api_lib::security::AuthenticatedUser;

use crate::database::project::delete::query::delete_project_query;
use crate::database::project::delete::view::DeleteProjectQueryView;
use crate::endpoints::v1::projects::project_id::ProjectPathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum DeleteProjectError {
    DatabaseError,
    UnknownProject,
}

impl std::fmt::Display for DeleteProjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeleteProjectError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            DeleteProjectError::UnknownProject => {
                write!(f, "Unknown project.")
            }
        }
    }
}

impl ResponseError for DeleteProjectError {
    fn status_code(&self) -> StatusCode {
        match self {
            DeleteProjectError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            DeleteProjectError::UnknownProject => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_delete_project(
    state: web::Data<AppState>,
    project_id: u64,
) -> Result<(), DeleteProjectError> {
    let pool = match state.db_pool.clone() {
        Some(pool) => pool,
        None => return Err(DeleteProjectError::DatabaseError),
    };

    let view = DeleteProjectQueryView::new(project_id);
    let result = delete_project_query(view, pool)
        .await
        .map_err(|_| DeleteProjectError::DatabaseError)?;

    // update cache

    if result == 0 {
        return Err(DeleteProjectError::UnknownProject);
    }

    Ok(())
}

#[utoipa::path(
    delete,
    path = "",
    responses(
        (status = 204, description = "Project deleted successfully"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ProjectPathParams
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Projects",
)]
#[delete("/")]
pub async fn delete_project(
    state: web::Data<AppState>,
    _: AuthenticatedUser,
    params: web::Path<ProjectPathParams>,
) -> Result<impl Responder, DeleteProjectError> {
    let project_id = params.project_id();
    trigger_delete_project(state, project_id).await?;
    Ok(HttpResponse::NoContent().finish())
}
