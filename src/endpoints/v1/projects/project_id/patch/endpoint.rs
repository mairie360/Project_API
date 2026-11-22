use actix_web::http::StatusCode;
use actix_web::{patch, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::smart_db::SmartTransaction;
use mairie360_api_lib::state::AppState;

use crate::endpoints::db_error::log_db_error;

use crate::database::project::group::view::{LinkableGroupQueryView, SetProjectGroupQueryView};
use crate::database::project::update::view::UpdateProjectQueryView;
use crate::endpoints::v1::projects::access::{begin_write, commit, AccessDenied, Requirement};
use crate::endpoints::v1::projects::get::view::stored_priority;
use crate::endpoints::v1::projects::project_id::patch::view::UpdateProjectView;
use crate::endpoints::v1::projects::project_id::ProjectPathParams;
use crate::endpoints::validation::ValidatedJson;
use mairie360_api_lib::error::ApiLibError;

#[derive(Debug, Clone, PartialEq)]
pub enum UpdateProjectError {
    Forbidden,
    BadRequest,
    NotFound,
    /// `group_id` names no group, or a group the caller may not link.
    UnknownGroup,
    DatabaseError,
}

impl std::fmt::Display for UpdateProjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdateProjectError::Forbidden => write!(f, "Forbidden."),
            UpdateProjectError::BadRequest => write!(f, "Bad request."),
            UpdateProjectError::NotFound => write!(f, "Unknown project."),
            UpdateProjectError::UnknownGroup => write!(f, "Unknown group."),
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
            UpdateProjectError::NotFound | UpdateProjectError::UnknownGroup => {
                StatusCode::NOT_FOUND
            }
            UpdateProjectError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_update_project(
    tx: &mut SmartTransaction,
    user_id: u64,
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
        Some(status) => Some(status.stored().ok_or(UpdateProjectError::BadRequest)?),
        None => None,
    };
    let mut query_view = UpdateProjectQueryView::new(
        project_id,
        view.name.as_deref().map(str::trim),
        view.description.as_deref(),
        status,
    );
    if view.budget_voted.is_some() || view.budget_committed.is_some() {
        query_view = query_view.with_budget(view.budget_voted, view.budget_committed);
    }
    if let Some(priority) = &view.priority {
        query_view = query_view
            .with_priority(stored_priority(priority).ok_or(UpdateProjectError::BadRequest)?);
    }

    let updated: bool = tx.fetch_scalar(&query_view).await.map_err(|e| {
        log_db_error("projects/project_id/patch", &e);
        UpdateProjectError::DatabaseError
    })?;

    if !updated {
        return Err(UpdateProjectError::NotFound);
    }
    if let Some(group_id) = view.group_id {
        set_project_group(tx, user_id, project_id, group_id).await?;
    }
    Ok(())
}

/// Replaces the group linked to the project, or removes it with `None`. A group that does not
/// exist or that the caller may not link answers `404` and rolls the whole update back.
async fn set_project_group(
    tx: &mut SmartTransaction,
    user_id: u64,
    project_id: u64,
    group_id: Option<u64>,
) -> Result<(), UpdateProjectError> {
    let database_error = |e: ApiLibError| {
        log_db_error("projects/project_id/patch", &e);
        UpdateProjectError::DatabaseError
    };
    if let Some(group_id) = group_id {
        let linkable: bool = tx
            .fetch_scalar(&LinkableGroupQueryView::new(user_id, group_id))
            .await
            .map_err(database_error)?;
        if !linkable {
            return Err(UpdateProjectError::UnknownGroup);
        }
    }
    tx.execute(&SetProjectGroupQueryView::new(project_id, group_id))
        .await
        .map_err(database_error)
}

#[utoipa::path(
    patch,
    params(
        ProjectPathParams,
    ),
    path = "",
    summary = "Update a project",
    description = "Updates the name, the description, the status, the priority, the group or the \
                   budget of a project. \
                   Partial update: a missing or `null` field is kept, except `group_id`, which \
                   cannot be `null` (see below). Restricted to the managers of the project.\n\n\
                   A `name` made of spaces only is refused in `400`, and so is the `Error` status \
                   or priority, which is only a fallback on read and cannot be assigned. The \
                   priority is stored as given (MAIR-558), `Urgent` as `High`; it does not follow \
                   the tasks.\n\n\
                   `group_id` replaces the group linked to the project; `null` is refused in \
                   `400`, since every project keeps a group. The new group's \
                   current and future members see the project with the rights of a project \
                   member, the members of the former group lose it unless they are members of \
                   the project. The caller may link one of their own groups; Admin and Maire may \
                   link any group (`GET /api/v1/project-groups/`). Any other id answers `404` and \
                   changes nothing.\n\n\
                   `budget_voted` and `budget_committed` (MAIR-1085) are amounts in cents of euro: \
                   absent keeps an amount, `null` clears it, an integer from 0 replaces it; a \
                   negative amount answers `400`.\n\n\
                   The response has an empty body; read the project back with \
                   `GET /api/v1/projects/{project_id}/`.",
    request_body(
        content = UpdateProjectView,
        description = "Fields to change, all optional.",
        example = json!({ "status": "Review", "priority": "High", "group_id": 3, "budget_voted": 1250000, "budget_committed": 480000 })
    ),
    responses(
        (
            status = 204,
            description = "Project updated. Empty body.",
        ),
        (
            status = 400,
            description = "Malformed JSON body, `project_id` not an integer, `name` empty, `status` or `priority` not one of the enum values, `Error` status or priority, `group_id` set to `null`, or a field breaking its rules: `name` 1 to 255 characters, not blank, no control character; `description` at most 5000 characters, no control character other than line breaks and tabs; `budget_voted` / `budget_committed` negative, not an integer or above 2^63 - 1.",
            body = String,
            content_type = "text/plain",
            example = json!("Invalid `name`: must be at most 255 characters")
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
            description = "Project visible to the caller, but insufficient rights for this operation.",
            body = String,
            content_type = "text/plain",
            example = json!("Forbidden.")
        ),
        (
            status = 404,
            description = "Project that does not exist or that the caller cannot see (`Unknown project.`), or `group_id` naming no group or a group the caller may not link (`Unknown group.`). Nothing is changed.",
            body = String,
            content_type = "text/plain",
            example = json!("Unknown project.")
        ),
        (
            status = 500,
            description = "Database error.",
            body = String,
            content_type = "text/plain",
            example = json!("An error occurred while accessing the database.")
        ),
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
    view: ValidatedJson<UpdateProjectView>,
) -> Result<impl Responder, UpdateProjectError> {
    let (mut tx, _) = begin_write(
        &state,
        auth_user.id,
        params.project_id(),
        None,
        Requirement::ManageProject,
    )
    .await?;
    trigger_update_project(
        &mut tx,
        auth_user.id,
        params.project_id(),
        view.into_inner(),
    )
    .await?;
    commit(tx, "projects/project_id/patch").await?;
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
