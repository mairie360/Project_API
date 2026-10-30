use actix_web::http::StatusCode;
use actix_web::{patch, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::smart_db::SmartTransaction;
use mairie360_api_lib::state::AppState;

use crate::endpoints::db_error::log_db_error;

use crate::database::project::update::view::UpdateProjectQueryView;
use crate::endpoints::v1::projects::access::{begin_write, commit, AccessDenied, Requirement};
use crate::endpoints::v1::projects::get::view::stored_priority;
use crate::endpoints::v1::projects::project_id::patch::view::UpdateProjectView;
use crate::endpoints::v1::projects::project_id::ProjectPathParams;
use crate::endpoints::validation::ValidatedJson;

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
    tx: &mut SmartTransaction,
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
    if let Some(priority) = &view.priority {
        query_view = query_view
            .with_priority(stored_priority(priority).ok_or(UpdateProjectError::BadRequest)?);
    }

    let updated: bool = tx.fetch_scalar(&query_view).await.map_err(|e| {
        log_db_error("projects/project_id/patch", &e);
        UpdateProjectError::DatabaseError
    })?;

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
    summary = "Update a project",
    description = "Updates the name, the description, the status or the priority of a project. \
                   Partial update: a missing or `null` field is kept. Restricted to the managers \
                   of the project.\n\n\
                   A `name` made of spaces only is refused in `400`, and so is the `Error` status \
                   or priority, which is only a fallback on read and cannot be assigned. The \
                   priority is stored as given (MAIR-558), `Urgent` as `High`; it does not follow \
                   the tasks.\n\n\
                   The response has an empty body; read the project back with \
                   `GET /api/v1/projects/{project_id}/`.",
    request_body(
        content = UpdateProjectView,
        description = "Fields to change, all optional.",
        example = json!({ "status": "Review", "priority": "High" })
    ),
    responses(
        (
            status = 204,
            description = "Projet mis à jour. Corps vide.",
        ),
        (
            status = 400,
            description = "Malformed JSON body, `project_id` not an integer, `name` empty, `status` or `priority` not one of the enum values, `Error` status or priority, or a field breaking its rules: `name` 1 to 255 characters, not blank, no control character; `description` at most 5000 characters, no control character other than line breaks and tabs.",
            body = String,
            content_type = "text/plain",
            example = json!("Invalid `name`: must be at most 255 characters")
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
            description = "Projet visible par l'appelant, mais droits insuffisants pour cette opération.",
            body = String,
            content_type = "text/plain",
            example = json!("Forbidden.")
        ),
        (
            status = 404,
            description = "Projet inexistant, ou invisible pour l'appelant.",
            body = String,
            content_type = "text/plain",
            example = json!("Unknown project.")
        ),
        (
            status = 500,
            description = "Erreur de base de données.",
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
    trigger_update_project(&mut tx, params.project_id(), view.into_inner()).await?;
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
