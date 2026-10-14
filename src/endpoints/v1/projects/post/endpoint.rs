use actix_web::http::StatusCode;
use actix_web::{post, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::database::db_interface::id_from_sql;
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::endpoints::db_error::log_db_error;

use crate::database::project::create::view::CreateProjectQueryView;
use crate::endpoints::v1::projects::access::{require_manager_role, AccessDenied};
use crate::endpoints::v1::projects::get::view::stored_priority;
use crate::endpoints::v1::projects::post::view::{CreateProjectResultView, CreateProjectView};
use crate::endpoints::validation::ValidatedJson;

#[derive(Debug, Clone, PartialEq)]
pub enum CreateProjectError {
    Forbidden,
    BadRequest,
    NotFound,
    DatabaseError,
}

impl std::fmt::Display for CreateProjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CreateProjectError::Forbidden => write!(f, "Forbidden."),
            CreateProjectError::BadRequest => write!(f, "Bad request."),
            CreateProjectError::NotFound => write!(f, "Not found."),
            CreateProjectError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for CreateProjectError {
    fn status_code(&self) -> StatusCode {
        match self {
            CreateProjectError::Forbidden => StatusCode::FORBIDDEN,
            CreateProjectError::BadRequest => StatusCode::BAD_REQUEST,
            CreateProjectError::NotFound => StatusCode::NOT_FOUND,
            CreateProjectError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
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
) -> Result<i32, CreateProjectError> {
    let mut query_view = CreateProjectQueryView::new(view.name(), view.description(), user_id);
    if let Some(status) = view.status() {
        query_view = query_view.with_status(status.stored().ok_or(CreateProjectError::BadRequest)?);
    }
    if let Some(priority) = view.priority() {
        query_view = query_view
            .with_priority(stored_priority(priority).ok_or(CreateProjectError::BadRequest)?);
    }
    let view = query_view;
    let result: i32 = state
        .get_smart_db()
        .fetch_scalar::<i32, _>(&view)
        .await
        .map_err(|e| {
            log_db_error("projects/post", &e);
            CreateProjectError::DatabaseError
        })?;

    Ok(result)
}

#[utoipa::path(
    post,
    path = "",
    summary = "Create a project",
    description = "Creates a project the caller becomes responsible for. Restricted to the Admin, \
                   Maire and Responsable roles: an agent without one of them gets `403`.\n\n\
                   Attaching a `group_id` gives every member of that group, as Core API defines \
                   it, access to the project right away.\n\n\
                   `status` and `priority` are stored as given (MAIR-558): a project created \
                   `Todo` / `High` is read back `Todo` / `High`. Without them the project is \
                   `Active` / `Medium`. `Urgent` is stored `High`; `Error` is refused in `400` for \
                   both.\n\n\
                   The response only holds the new id; read the whole project back with \
                   `GET /api/v1/projects/{project_id}/`.",
    responses(
        (
            status = 200,
            description = "Projet créé. Le corps contient l'identifiant attribué.",
            body = CreateProjectResultView,
            example = json!({ "project_id": 12 })
        ),
        (
            status = 400,
            description = "Malformed JSON body, missing `name`, `status` or `priority` not one of the enum values, `Error` as `status` or `priority`, or a field breaking its rules: `name` 1 to 255 characters, not blank, no control character; `description` at most 5000 characters, no control character other than line breaks and tabs. The body names the first invalid field, except for `Error` (`Bad request.`).",
            body = String,
            content_type = "text/plain",
            example = json!("Invalid `name`: must not contain control characters")
        ),
        (
            status = 401,
            description = "En-tête `Authorization` absent, JWT invalide ou expiré, ou session révoquée.",
            body = String,
            content_type = "text/plain",
            example = json!("Jeton expiré")
        ),
        (
            status = 403,
            description = "L'appelant n'a ni le rôle Admin, ni Maire, ni Responsable.",
            body = String,
            content_type = "text/plain",
            example = json!("Forbidden.")
        ),
        (
            status = 500,
            description = "Erreur de base de données.",
            body = String,
            content_type = "text/plain",
            example = json!("An error occurred while accessing the database.")
        ),
    ),
    request_body(
        content = CreateProjectView,
        description = "Project name; description, group, status and priority are optional.",
        example = json!({
            "name": "Réfection de la place du marché",
            "description": "Travaux de voirie 2026",
            "group_id": 3,
            "status": "Todo",
            "priority": "High"
        })
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Projects",
)]
#[post("/")]
pub async fn create_project(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    view: ValidatedJson<CreateProjectView>,
) -> Result<impl Responder, CreateProjectError> {
    require_manager_role(&state, auth_user.id).await?;
    let view = view.into_inner();
    let project_id = trigger_create_project(state, auth_user.id, view).await?;
    Ok(HttpResponse::Ok().json(CreateProjectResultView {
        project_id: id_from_sql(project_id),
    }))
}

impl From<AccessDenied> for CreateProjectError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => CreateProjectError::NotFound,
            AccessDenied::Forbidden => CreateProjectError::Forbidden,
            AccessDenied::DatabaseError => CreateProjectError::DatabaseError,
        }
    }
}
