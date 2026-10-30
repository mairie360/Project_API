//! Project templates (MAIR-566): read by any signed-in user, managed by Admin and Responsable.
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};

pub mod doc;
pub mod endpoint;
pub mod view;

/// Path parameters of the routes of one template.
#[derive(serde::Deserialize, utoipa::IntoParams, utoipa::ToSchema)]
#[into_params(parameter_in = Path)]
pub struct TemplatePathParams {
    /// Id of the template.
    #[param(example = 2)]
    #[schema(example = 2)]
    template_id: u64,
}

impl TemplatePathParams {
    pub fn new(template_id: u64) -> Self {
        Self { template_id }
    }

    pub fn template_id(&self) -> u64 {
        self.template_id
    }
}

/// Failure of a template route.
#[derive(Debug, Clone, PartialEq)]
pub enum TemplatesError {
    /// The caller is neither Admin nor Responsable.
    Forbidden,
    NotFound,
    /// Another template has this name, whatever the case.
    NameTaken,
    /// The default template cannot be deleted.
    DefaultTemplate,
    DatabaseError,
}

impl std::fmt::Display for TemplatesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemplatesError::Forbidden => write!(f, "Forbidden."),
            TemplatesError::NotFound => write!(f, "Unknown template."),
            TemplatesError::NameTaken => write!(f, "Another template already has this name."),
            TemplatesError::DefaultTemplate => {
                write!(f, "The default template cannot be deleted.")
            }
            TemplatesError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for TemplatesError {
    fn status_code(&self) -> StatusCode {
        match self {
            TemplatesError::Forbidden => StatusCode::FORBIDDEN,
            TemplatesError::NotFound => StatusCode::NOT_FOUND,
            TemplatesError::NameTaken | TemplatesError::DefaultTemplate => StatusCode::CONFLICT,
            TemplatesError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/project-templates")
            .service(endpoint::get_templates)
            .service(endpoint::create_template)
            .service(endpoint::get_template)
            .service(endpoint::update_template)
            .service(endpoint::delete_template),
    );
}
