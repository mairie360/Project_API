use actix_web::http::StatusCode;
use actix_web::{post, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::database::db_interface::id_from_sql;
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::project::group::view::LinkableGroupQueryView;
use crate::endpoints::db_error::{
    classify_db_error, log_db_error, violated_foreign_key, DbFailure,
};

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
    UnknownTemplate,
    /// `responsible_id` names no user.
    UnknownResponsible,
    /// `group_id` names no group, or a group the caller may not link.
    UnknownGroup,
    DatabaseError,
}

impl std::fmt::Display for CreateProjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CreateProjectError::Forbidden => write!(f, "Forbidden."),
            CreateProjectError::BadRequest => write!(f, "Bad request."),
            CreateProjectError::NotFound => write!(f, "Not found."),
            CreateProjectError::UnknownTemplate => {
                write!(f, "`template_id` does not match any project template.")
            }
            CreateProjectError::UnknownResponsible => {
                write!(f, "`responsible_id` does not match any user.")
            }
            CreateProjectError::UnknownGroup => write!(f, "Unknown group."),
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
            CreateProjectError::NotFound | CreateProjectError::UnknownGroup => {
                StatusCode::NOT_FOUND
            }
            CreateProjectError::UnknownTemplate | CreateProjectError::UnknownResponsible => {
                StatusCode::BAD_REQUEST
            }
            CreateProjectError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

/// Refuses a group that does not exist or that the caller may not link (not one of their groups,
/// unless they are Admin or Maire) with `404`, so that the existence of other groups is not
/// revealed.
async fn require_linkable_group(
    state: &web::Data<AppState>,
    user_id: u64,
    group_id: u64,
) -> Result<(), CreateProjectError> {
    let linkable: bool = state
        .get_smart_db()
        .fetch_scalar(&LinkableGroupQueryView::new(user_id, group_id))
        .await
        .map_err(|e| {
            log_db_error("projects/post", &e);
            CreateProjectError::DatabaseError
        })?;
    if linkable {
        Ok(())
    } else {
        Err(CreateProjectError::UnknownGroup)
    }
}

/// Whether the broken foreign key is the responsible's (`projects.responsible_id` or the member row
/// written for them), not the template's.
fn references_a_user(error: &mairie360_api_lib::error::ApiLibError) -> bool {
    violated_foreign_key(error)
        .is_some_and(|constraint| constraint.contains("responsible") || constraint.contains("user"))
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
    if let Some(group_id) = view.group_id() {
        require_linkable_group(&state, user_id, group_id).await?;
    }
    let view = query_view
        .with_template(view.template_id())
        .with_group(view.group_id())
        .with_responsible(view.responsible_id());
    let result: i32 = state
        .get_smart_db()
        .fetch_scalar::<i32, _>(&view)
        .await
        .map_err(|e| match classify_db_error("projects/post", &e) {
            DbFailure::InvalidReference if references_a_user(&e) => {
                CreateProjectError::UnknownResponsible
            }
            DbFailure::InvalidReference => CreateProjectError::UnknownTemplate,
            _ => CreateProjectError::DatabaseError,
        })?;

    Ok(result)
}

#[utoipa::path(
    post,
    path = "",
    summary = "Create a project",
    description = "Creates a project the caller becomes responsible for. Restricted to the Admin, \
                   Maire and Responsable roles: an agent without one of them gets `403`.\n\n\
                   `group_id` is required (`400` without it): every project has a group. It \
                   gives every current and future member of that group, as \
                   Core API defines it, access to the project right away, with the rights of a \
                   project member. The caller may link one of their own groups; Admin and Maire \
                   may link any group (`GET /api/v1/project-groups/` lists them). Any other id \
                   answers `404` and creates nothing. Change the group later with \
                   `PATCH /api/v1/projects/{project_id}/`.\n\n\
                   `status` and `priority` are stored as given (MAIR-558): a project created \
                   `Todo` / `High` is read back `Todo` / `High`. Without them the project is \
                   `Active` / `Medium`. `Urgent` is stored `High`; `Error` is refused in `400` for \
                   both.\n\n\
                   `template_id` (`GET /api/v1/project-templates/`, MAIR-566) gives the project a \
                   copy of the template's task statuses and custom fields; without it, the default \
                   template's. Editing the template later never changes the project.\n\n\
                   `responsible_id` records the responsible of the project, read back as \
                   `responsible` by the project reads: that user becomes a `manager` member in the \
                   same write. Without it the project has no responsible (`null`). The caller stays \
                   the owner either way.\n\n\
                   The response only holds the new id; read the whole project back with \
                   `GET /api/v1/projects/{project_id}/`.",
    responses(
        (
            status = 200,
            description = "Project created. The body holds the assigned id.",
            body = CreateProjectResultView,
            example = json!({ "project_id": 12 })
        ),
        (
            status = 400,
            description = "Malformed JSON body, missing `name` or `group_id`, `status` or `priority` not one of the enum values, `Error` as `status` or `priority`, `template_id` that matches no project template, `responsible_id` that matches no user, or a field breaking its rules: `name` 1 to 255 characters, not blank, no control character; `description` at most 5000 characters, no control character other than line breaks and tabs. The body names the first invalid field, except for `Error` (`Bad request.`).",
            body = String,
            content_type = "text/plain",
            example = json!("Invalid `name`: must not contain control characters")
        ),
        (
            status = 401,
            description = "Missing `Authorization` header, invalid or expired JWT, or revoked session.",
            body = String,
            content_type = "text/plain",
            example = json!("Jeton expiré")
        ),
        (
            status = 403,
            description = "The caller has none of the Admin, Maire and Responsable roles.",
            body = String,
            content_type = "text/plain",
            example = json!("Forbidden.")
        ),
        (
            status = 404,
            description = "`group_id` names no group, or a group the caller may not link (not one of their groups, unless they are Admin or Maire). Nothing is created.",
            body = String,
            content_type = "text/plain",
            example = json!("Unknown group.")
        ),
        (
            status = 500,
            description = "Database error.",
            body = String,
            content_type = "text/plain",
            example = json!("An error occurred while accessing the database.")
        ),
    ),
    request_body(
        content = CreateProjectView,
        description = "Project name and group; description, status, priority, template and responsible are optional.",
        example = json!({
            "name": "Renovation of the market square",
            "description": "2026 road works",
            "group_id": 3,
            "status": "Todo",
            "priority": "High",
            "responsible_id": 7
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
