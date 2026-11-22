//! Milestones of a project (MAIR-1086): listed by anyone who sees the project, managed by its
//! managers.
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};

use crate::endpoints::v1::projects::access::AccessDenied;

pub mod doc;
pub mod endpoint;
pub mod view;

/// Path parameters of the routes of one milestone of a project.
#[derive(serde::Deserialize, utoipa::IntoParams, utoipa::ToSchema)]
#[into_params(parameter_in = Path)]
pub struct ProjectMilestonePathParams {
    /// Id of the project.
    #[param(example = 12)]
    #[schema(example = 12)]
    project_id: u64,
    /// Id of the milestone, among the project's milestones.
    #[param(example = 5)]
    #[schema(example = 5)]
    milestone_id: u64,
}

impl ProjectMilestonePathParams {
    pub fn new(project_id: u64, milestone_id: u64) -> Self {
        Self {
            project_id,
            milestone_id,
        }
    }

    pub fn project_id(&self) -> u64 {
        self.project_id
    }

    pub fn milestone_id(&self) -> u64 {
        self.milestone_id
    }
}

/// Failure of a route of the project milestones.
#[derive(Debug, Clone, PartialEq)]
pub enum MilestonesError {
    Forbidden,
    /// Unknown or invisible project.
    NotFound,
    /// No such milestone in the project, or a deleted one.
    UnknownMilestone,
    /// The database refused a value (a check constraint).
    BadRequest,
    DatabaseError,
}

impl std::fmt::Display for MilestonesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MilestonesError::Forbidden => write!(f, "Forbidden."),
            MilestonesError::NotFound => write!(f, "Not found."),
            MilestonesError::UnknownMilestone => write!(f, "Unknown milestone."),
            MilestonesError::BadRequest => write!(f, "Invalid milestone."),
            MilestonesError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for MilestonesError {
    fn status_code(&self) -> StatusCode {
        match self {
            MilestonesError::Forbidden => StatusCode::FORBIDDEN,
            MilestonesError::NotFound | MilestonesError::UnknownMilestone => StatusCode::NOT_FOUND,
            MilestonesError::BadRequest => StatusCode::BAD_REQUEST,
            MilestonesError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

impl From<AccessDenied> for MilestonesError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => MilestonesError::NotFound,
            AccessDenied::Forbidden => MilestonesError::Forbidden,
            AccessDenied::DatabaseError => MilestonesError::DatabaseError,
        }
    }
}

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/milestones")
            .service(endpoint::get_project_milestones)
            .service(endpoint::create_project_milestone)
            .service(endpoint::update_project_milestone)
            .service(endpoint::delete_project_milestone),
    );
}
