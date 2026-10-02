use actix_web::http::StatusCode;
use actix_web::{delete, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::smart_db::SmartTransaction;
use mairie360_api_lib::state::AppState;

use crate::endpoints::db_error::log_db_error;

use crate::database::users::remove_user_from_project::view::{
    RemoveUserFromProjectQueryView, UnassignMemberTasksQueryView,
};
use crate::endpoints::v1::projects::access::{begin_write, commit, AccessDenied, Requirement};
use crate::endpoints::v1::projects::project_id::users::user_id::ProjectUserPathParams;

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum RemoveUserFromProjectError {
    Forbidden,
    NotFound,
    DatabaseError,
    UnknownUser,
}

impl std::fmt::Display for RemoveUserFromProjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RemoveUserFromProjectError::Forbidden => write!(f, "Forbidden."),
            RemoveUserFromProjectError::NotFound => write!(f, "Not found."),
            RemoveUserFromProjectError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            RemoveUserFromProjectError::UnknownUser => {
                write!(f, "Unknown user.")
            }
        }
    }
}

impl ResponseError for RemoveUserFromProjectError {
    fn status_code(&self) -> StatusCode {
        match self {
            RemoveUserFromProjectError::Forbidden => StatusCode::FORBIDDEN,
            RemoveUserFromProjectError::NotFound => StatusCode::NOT_FOUND,
            RemoveUserFromProjectError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            RemoveUserFromProjectError::UnknownUser => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_remove_user_from_project(
    tx: &mut SmartTransaction,
    project_id: u64,
    user_id: u64,
    caller_id: u64,
) -> Result<(), RemoveUserFromProjectError> {
    let log = |e: &dyn std::fmt::Display| {
        log_db_error("projects/project_id/users/user_id/delete", &e.to_string());
        RemoveUserFromProjectError::DatabaseError
    };
    tx.execute(&RemoveUserFromProjectQueryView::new(project_id, user_id))
        .await
        .map_err(|e| log(&e))?;
    // Same transaction: the member and their assignments go together.
    tx.execute(&UnassignMemberTasksQueryView::new(
        project_id, user_id, caller_id,
    ))
    .await
    .map_err(|e| log(&e))?;

    Ok(())
}

#[utoipa::path(
    delete,
    path = "",
    summary = "Remove a member from a project",
    description = "Detaches a user from the project. The account and the project are kept; only \
                   the membership goes, and the project no longer appears in that user's \
                   `GET /api/v1/projects/`. Reserved to the project's managers.\n\n\
                   In the same transaction, the tasks of this project assigned to the user are \
                   unassigned (`assigned_to` becomes `null`, logged in each task's history and \
                   signed by the caller), since a task can only be assigned to the owner or a \
                   member of its project. Nothing changes when the user is the project's owner.\n\n\
                   Idempotent once the rights are checked: removing someone who is not a member \
                   also answers `204`.",
    responses(
        (
            status = 204,
            description = "Utilisateur retiré du projet. Corps vide.",
        ),
        (
            status = 400,
            description = "Un segment de l'URL n'est pas un entier, ou le corps JSON est malformé.",
            body = String,
            content_type = "text/plain",
            example = json!("Path deserialize error: can not parse `abc` to a u64")
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
            description = "Projet inexistant, ou invisible pour l'appelant — les deux cas sont volontairement indiscernables.",
            body = String,
            content_type = "text/plain",
            example = json!("Not found.")
        ),
        (
            status = 500,
            description = "Erreur de base de données.",
            body = String,
            content_type = "text/plain",
            example = json!("An error occurred while accessing the database.")
        ),
    ),
    params(
        ProjectUserPathParams,
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Users",
)]
#[delete("/")]
pub async fn remove_user_from_project(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<ProjectUserPathParams>,
) -> Result<impl Responder, RemoveUserFromProjectError> {
    let (mut tx, _) = begin_write(
        &state,
        auth_user.id,
        params.project_id(),
        None,
        Requirement::ManageProject,
    )
    .await?;
    let project_id = params.project_id();
    let user_id = params.user_id();
    trigger_remove_user_from_project(&mut tx, project_id, user_id, auth_user.id).await?;
    commit(tx, "projects/project_id/users/user_id/delete").await?;
    Ok(HttpResponse::NoContent().finish())
}

impl From<AccessDenied> for RemoveUserFromProjectError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => RemoveUserFromProjectError::NotFound,
            AccessDenied::Forbidden => RemoveUserFromProjectError::Forbidden,
            AccessDenied::DatabaseError => RemoveUserFromProjectError::DatabaseError,
        }
    }
}
