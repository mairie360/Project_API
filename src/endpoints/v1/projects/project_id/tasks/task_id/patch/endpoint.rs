use actix_web::http::StatusCode;
use actix_web::{patch, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::smart_db::SmartTransaction;
use mairie360_api_lib::state::AppState;

use crate::endpoints::db_error::{classify, DbFailure};

use crate::database::tasks::create_task::view::{
    TaskPriority as DbTaskPriority, TaskStatus as DbTaskStatus,
};
use crate::database::tasks::patch_task::view::{PatchTaskQueryView, TaskChanges};
use crate::endpoints::v1::projects::access::{begin_write, commit, AccessDenied, Requirement};
use crate::endpoints::v1::projects::project_id::get::view::TaskPriority;
use crate::endpoints::v1::projects::project_id::tasks::task_id::patch::view::PatchTaskView;
use crate::endpoints::v1::projects::project_id::tasks::task_id::TaskPathParams;
use crate::endpoints::validation::ValidatedJson;

#[derive(Debug, Clone, PartialEq)]
pub enum PatchTaskError {
    Forbidden,
    BadRequest,
    NotFound,
    DatabaseError,
    UnknownAssignee,
    AssigneeNotInProject,
}

impl std::fmt::Display for PatchTaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PatchTaskError::Forbidden => write!(f, "Forbidden."),
            PatchTaskError::BadRequest => write!(f, "Bad request."),
            PatchTaskError::NotFound => write!(f, "Unknown task."),
            PatchTaskError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            PatchTaskError::UnknownAssignee => write!(f, "`assigned_to` does not match any user."),
            PatchTaskError::AssigneeNotInProject => {
                write!(f, "`assigned_to` is not a member of the project.")
            }
        }
    }
}

impl ResponseError for PatchTaskError {
    fn status_code(&self) -> StatusCode {
        match self {
            PatchTaskError::Forbidden => StatusCode::FORBIDDEN,
            PatchTaskError::BadRequest => StatusCode::BAD_REQUEST,
            PatchTaskError::NotFound => StatusCode::NOT_FOUND,
            PatchTaskError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            PatchTaskError::UnknownAssignee => StatusCode::BAD_REQUEST,
            PatchTaskError::AssigneeNotInProject => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_patch_task(
    tx: &mut SmartTransaction,
    project_id: u64,
    task_id: u64,
    user_id: u64,
    view: PatchTaskView,
) -> Result<(), PatchTaskError> {
    if view
        .name
        .as_deref()
        .is_some_and(|name| name.trim().is_empty())
    {
        return Err(PatchTaskError::BadRequest);
    }
    let status = view
        .status
        .map(|status| DbTaskStatus::from(status.to_string()));
    // The database has no "urgent" priority: the highest one is used.
    let priority = view.priority.map(|priority| match priority {
        TaskPriority::Urgent => DbTaskPriority::High,
        priority => DbTaskPriority::from(priority.to_string()),
    });
    if status == Some(DbTaskStatus::Error) || priority == Some(DbTaskPriority::Error) {
        return Err(PatchTaskError::BadRequest);
    }

    let updated: bool = tx
        .fetch_scalar(&PatchTaskQueryView::new(
            project_id,
            task_id,
            user_id,
            TaskChanges {
                title: view.name.as_deref().map(str::trim),
                description: view.description.as_deref(),
                status,
                priority,
                due_date: view.due_date,
                assigned_to: view.assigned_to,
                fields: view.fields.as_deref(),
            },
        ))
        .await
        .map_err(
            |e| match classify("projects/project_id/tasks/task_id/patch", &e) {
                DbFailure::InvalidReference => PatchTaskError::UnknownAssignee,
                _ => PatchTaskError::DatabaseError,
            },
        )?;

    match (updated, view.assigned_to) {
        (true, _) => Ok(()),
        // The task exists (checked by `require_access`): the new assignee was refused.
        (false, Some(Some(_))) => Err(PatchTaskError::AssigneeNotInProject),
        (false, _) => Err(PatchTaskError::NotFound),
    }
}

#[utoipa::path(
    patch,
    path = "",
    summary = "Update a task",
    description = "Partially updates a task: an absent field is left unchanged.\n\n\
                   Two levels of rights apply. A **project manager** can change everything. An \
                   **agent assigned to the task** can only change its status: if the body touches \
                   anything else, the answer is `403`.\n\n\
                   `assigned_to` tells absence from `null`: omitting it keeps the assignee, \
                   sending `null` removes it. A new assignee must be the owner or a member of the \
                   project. `description` replaces the stored description (send `\"\"` to clear \
                   it); `fields` replaces the whole list of custom fields.\n\n\
                   Every change is logged server-side and signed by the caller: one \
                   `status_changed` entry for a status change, one `task_updated` entry listing \
                   the other changed fields (see `GET …/tasks/{task_id}/collaboration`). The \
                   response has an empty body.",
    responses(
        (
            status = 204,
            description = "Task updated. Empty body.",
        ),
        (
            status = 400,
            description = "A URL segment is not an integer, malformed JSON body, `assigned_to` that does not match any user or is neither the owner nor a member of the project, or a field breaking its rules: `name` 1 to 255 characters, not blank, no control character, no `<` or `>`; `description` at most 5000 characters, no `<` or `>`, no control character other than line breaks and tabs; each `fields[].label` 1 to 255 characters (same rules as `name`) and each `fields_options[].option` string free of `<`, `>` and control characters.",
            body = String,
            content_type = "text/plain",
            example = json!("`assigned_to` is not a member of the project.")
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
            description = "Insufficient rights on the task, or an assigned agent trying to change something else than the status.",
            body = String,
            content_type = "text/plain",
            example = json!("Forbidden.")
        ),
        (
            status = 404,
            description = "Unknown project or task, or project not visible to the caller.",
            body = String,
            content_type = "text/plain",
            example = json!("Unknown task.")
        ),
        (
            status = 500,
            description = "Database error.",
            body = String,
            content_type = "text/plain",
            example = json!("An error occurred while accessing the database.")
        ),
    ),
    params(
        TaskPathParams
    ),
    request_body(
        content = PatchTaskView,
        description = "Fields to change. All optional.",
        example = json!({ "status": "Completed" })
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Tasks",
)]
#[patch("/")]
pub async fn patch_task(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<TaskPathParams>,
    view: ValidatedJson<PatchTaskView>,
) -> Result<impl Responder, PatchTaskError> {
    let view = view.into_inner();
    let (mut tx, access) = begin_write(
        &state,
        auth_user.id,
        params.project_id(),
        Some(params.task_id()),
        Requirement::ActOnTask,
    )
    .await?;
    // An assigned agent who does not manage the project may only change the status of their task.
    if !access.can_manage() && !view.only_changes_status() {
        return Err(PatchTaskError::Forbidden);
    }
    trigger_patch_task(
        &mut tx,
        params.project_id(),
        params.task_id(),
        auth_user.id,
        view,
    )
    .await?;
    commit(tx, "projects/project_id/tasks/task_id/patch").await?;
    Ok(HttpResponse::NoContent().finish())
}

impl From<AccessDenied> for PatchTaskError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => PatchTaskError::NotFound,
            AccessDenied::Forbidden => PatchTaskError::Forbidden,
            AccessDenied::DatabaseError => PatchTaskError::DatabaseError,
        }
    }
}
