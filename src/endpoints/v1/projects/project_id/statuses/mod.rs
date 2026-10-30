//! Task statuses of a project (MAIR-566): listed by anyone who sees the project, managed by its
//! managers.
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};

use crate::endpoints::v1::projects::access::AccessDenied;

pub mod doc;
pub mod endpoint;
pub mod view;

/// Path parameters of the routes of one status of a project.
#[derive(serde::Deserialize, utoipa::IntoParams, utoipa::ToSchema)]
#[into_params(parameter_in = Path)]
pub struct ProjectStatusPathParams {
    /// Id of the project.
    #[param(example = 12)]
    #[schema(example = 12)]
    project_id: u64,
    /// Id of the status, among the project's statuses.
    #[param(example = 31)]
    #[schema(example = 31)]
    status_id: u64,
}

impl ProjectStatusPathParams {
    pub fn new(project_id: u64, status_id: u64) -> Self {
        Self {
            project_id,
            status_id,
        }
    }

    pub fn project_id(&self) -> u64 {
        self.project_id
    }

    pub fn status_id(&self) -> u64 {
        self.status_id
    }
}

/// Failure of a route of the project statuses.
#[derive(Debug, Clone, PartialEq)]
pub enum StatusesError {
    Forbidden,
    /// Unknown or invisible project.
    NotFound,
    /// No such status in the project.
    UnknownStatus,
    /// Another status of the project has this name, whatever the case.
    NameTaken,
    /// The change would leave a category without any status in the project.
    LastOfCategory,
    /// The status holds tasks and no `move_to` was given.
    TasksToMove,
    /// `move_to` is not another status of the project.
    InvalidTarget,
    DatabaseError,
}

impl std::fmt::Display for StatusesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatusesError::Forbidden => write!(f, "Forbidden."),
            StatusesError::NotFound => write!(f, "Not found."),
            StatusesError::UnknownStatus => write!(f, "Unknown status."),
            StatusesError::NameTaken => {
                write!(f, "Another status of the project already has this name.")
            }
            StatusesError::LastOfCategory => write!(
                f,
                "A project keeps at least one status of each category (Todo, InProgress, Completed)."
            ),
            StatusesError::TasksToMove => write!(
                f,
                "The status holds tasks: give `move_to`, the status to move them to."
            ),
            StatusesError::InvalidTarget => {
                write!(f, "`move_to` is not another status of the project.")
            }
            StatusesError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for StatusesError {
    fn status_code(&self) -> StatusCode {
        match self {
            StatusesError::Forbidden => StatusCode::FORBIDDEN,
            StatusesError::NotFound | StatusesError::UnknownStatus => StatusCode::NOT_FOUND,
            StatusesError::NameTaken
            | StatusesError::LastOfCategory
            | StatusesError::TasksToMove => StatusCode::CONFLICT,
            StatusesError::InvalidTarget => StatusCode::BAD_REQUEST,
            StatusesError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

impl From<AccessDenied> for StatusesError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => StatusesError::NotFound,
            AccessDenied::Forbidden => StatusesError::Forbidden,
            AccessDenied::DatabaseError => StatusesError::DatabaseError,
        }
    }
}

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/statuses")
            .service(endpoint::get_project_statuses)
            .service(endpoint::create_project_status)
            .service(endpoint::update_project_status)
            .service(endpoint::delete_project_status),
    );
}
