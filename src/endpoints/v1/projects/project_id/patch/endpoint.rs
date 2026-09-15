use actix_web::http::StatusCode;
use actix_web::{patch, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::project::update::view::UpdateProjectQueryView;
use crate::database::project::update_status::view::ProjectStatus as DbProjectStatus;
use crate::endpoints::v1::projects::access::{require_access, AccessDenied, Requirement};
use crate::endpoints::v1::projects::get::view::ProjectStatus;
use crate::endpoints::v1::projects::project_id::patch::view::UpdateProjectView;
use crate::endpoints::v1::projects::project_id::ProjectPathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum UpdateProjectError {
    Forbidden,
    BadRequest,
    NotFound,
    DatabaseError,
}

impl std::fmt::Display for UpdateProjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdateProjectError::Forbidden => write!(f, "Forbidden."),
            UpdateProjectError::BadRequest => write!(f, "Bad request."),
            UpdateProjectError::NotFound => write!(f, "Unknown project."),
            UpdateProjectError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for UpdateProjectError {
    fn status_code(&self) -> StatusCode {
        match self {
            UpdateProjectError::Forbidden => StatusCode::FORBIDDEN,
            UpdateProjectError::BadRequest => StatusCode::BAD_REQUEST,
            UpdateProjectError::NotFound => StatusCode::NOT_FOUND,
            UpdateProjectError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_update_project(
    state: web::Data<AppState>,
    project_id: u64,
    view: UpdateProjectView,
) -> Result<(), UpdateProjectError> {
    if view
        .name
        .as_deref()
        .is_some_and(|name| name.trim().is_empty())
    {
        return Err(UpdateProjectError::BadRequest);
    }
    let status = match view.status {
        Some(ProjectStatus::Error) => return Err(UpdateProjectError::BadRequest),
        Some(status) => Some(DbProjectStatus::from(status.to_string())),
        None => None,
    };

    let updated: bool = state
        .get_smart_db()
        .fetch_scalar(&UpdateProjectQueryView::new(
            project_id,
            view.name.as_deref().map(str::trim),
            view.description.as_deref(),
            status,
        ))
        .await
        .map_err(|_| UpdateProjectError::DatabaseError)?;

    if updated {
        Ok(())
    } else {
        Err(UpdateProjectError::NotFound)
    }
}

#[utoipa::path(
    patch,
    params(
        ProjectPathParams,
    ),
    path = "",
    request_body = UpdateProjectView,
    responses(
        (status = 204, description = "Project updated successfully"),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Unknown project"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Projects",
)]
#[patch("/")]
pub async fn update_project(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<ProjectPathParams>,
    view: web::Json<UpdateProjectView>,
) -> Result<impl Responder, UpdateProjectError> {
    require_access(
        &state,
        auth_user.id,
        params.project_id(),
        None,
        Requirement::ManageProject,
    )
    .await?;
    trigger_update_project(state, params.project_id(), view.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}

impl From<AccessDenied> for UpdateProjectError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => UpdateProjectError::NotFound,
            AccessDenied::Forbidden => UpdateProjectError::Forbidden,
            AccessDenied::DatabaseError => UpdateProjectError::DatabaseError,
        }
    }
}
