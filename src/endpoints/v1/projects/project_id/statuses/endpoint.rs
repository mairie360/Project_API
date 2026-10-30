use actix_web::{delete, get, patch, post, web, HttpResponse, Responder};
use mairie360_api_lib::database::db_interface::id_from_sql;
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::statuses::view::{
    CheckStatusDeletionQueryView, CreateProjectStatusQueryView, DeleteProjectStatusQueryView,
    ListProjectStatusesQueryView, MoveStatusTasksQueryView, ProjectStatusRow, StatusChanges,
    UpdateProjectStatusQueryView, DELETION_ALLOWED, DELETION_INVALID_TARGET,
    DELETION_LAST_OF_CATEGORY, DELETION_TASKS_TO_MOVE, STATUS_LAST_OF_CATEGORY, STATUS_UPDATED,
};
use crate::endpoints::db_error::{classify_db_error, log_db_error, DbFailure};
use crate::endpoints::v1::projects::access::{begin_write, commit, require_access, Requirement};
use crate::endpoints::v1::projects::project_id::statuses::view::{
    CreateProjectStatusResultView, CreateProjectStatusView, DeleteProjectStatusQuery,
    GetProjectStatusesResultView, PatchProjectStatusView, ProjectStatusView,
};
use crate::endpoints::v1::projects::project_id::statuses::{
    ProjectStatusPathParams, StatusesError,
};
use crate::endpoints::v1::projects::project_id::ProjectPathParams;
use crate::endpoints::validation::ValidatedJson;

/// A unique violation is a name taken; anything else is logged and becomes a `500`.
fn write_error(operation: &str, error: &mairie360_api_lib::error::ApiLibError) -> StatusesError {
    match classify_db_error(operation, error) {
        DbFailure::Conflict => StatusesError::NameTaken,
        _ => StatusesError::DatabaseError,
    }
}

#[utoipa::path(
    get,
    params(ProjectPathParams),
    path = "",
    summary = "List the task statuses of a project",
    description = "Returns the task statuses of the project by position (MAIR-566): the Kanban \
                   columns. They were copied from the project's template at its creation and are \
                   then the project's own. Each has a `category` (`Todo`, `InProgress`, \
                   `Completed`) that drives the archiving, the progress and the pending task \
                   lists, and the number of tasks in it. Being able to see the project is enough.",
    responses(
        (status = 200, description = "Statuses of the project, by position.", body = GetProjectStatusesResultView,
         example = json!({ "statuses": [
             { "id": 30, "name": "À faire", "color": "#64748B", "category": "Todo", "position": 0, "task_count": 3 },
             { "id": 31, "name": "En cours", "color": "#2563EB", "category": "InProgress", "position": 1, "task_count": 1 },
             { "id": 32, "name": "Terminé", "color": "#16A34A", "category": "Completed", "position": 2, "task_count": 5 }
         ] })),
        (status = 400, description = "A URL segment is not an integer.", body = String, content_type = "text/plain",
         example = json!("Path deserialize error: can not parse `abc` to a u64")),
        (status = 401, description = "Missing `Authorization` header, invalid or expired JWT, or revoked session.",
         body = String, content_type = "text/plain", example = json!("Jeton expiré")),
        (status = 404, description = "Unknown project, or project not visible to the caller — both cases are deliberately indistinguishable.",
         body = String, content_type = "text/plain", example = json!("Not found.")),
        (status = 500, description = "Database error.", body = String, content_type = "text/plain",
         example = json!("An error occurred while accessing the database.")),
    ),
    security(("jwt" = [])),
    tag = "Statuses",
)]
#[get("/")]
pub async fn get_project_statuses(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<ProjectPathParams>,
) -> Result<impl Responder, StatusesError> {
    require_access(
        &state,
        auth_user.id,
        params.project_id(),
        None,
        Requirement::ViewProject,
    )
    .await?;
    let rows: Vec<ProjectStatusRow> = state
        .get_smart_db()
        .fetch_one(&ListProjectStatusesQueryView::new(params.project_id()))
        .await
        .map_err(|e| {
            log_db_error("projects/project_id/statuses/get", &e);
            StatusesError::DatabaseError
        })?;
    Ok(HttpResponse::Ok().json(GetProjectStatusesResultView {
        statuses: rows
            .into_iter()
            .filter_map(ProjectStatusView::from_row)
            .collect(),
    }))
}

#[utoipa::path(
    post,
    params(ProjectPathParams),
    path = "",
    summary = "Add a task status to a project",
    description = "Adds a status (a Kanban column) to the project at `position`, the next ones \
                   moving down; at the end without it. Reserved to the project managers. The name \
                   must be unique in the project, whatever the case.",
    request_body(content = CreateProjectStatusView, example = json!({
        "name": "À valider", "color": "#D97706", "category": "InProgress", "position": 2
    })),
    responses(
        (status = 200, description = "Status added.", body = CreateProjectStatusResultView,
         example = json!({ "status_id": 33 })),
        (status = 400, description = "A URL segment is not an integer, malformed JSON body, `category` not one of the enum values, or a field breaking its rules: `name` 1 to 100 characters, not blank, no control character; `color` `#` followed by 6 hexadecimal digits.",
         body = String, content_type = "text/plain", example = json!("Invalid `color`: must be `#` followed by 6 hexadecimal digits")),
        (status = 401, description = "Missing `Authorization` header, invalid or expired JWT, or revoked session.",
         body = String, content_type = "text/plain", example = json!("Jeton expiré")),
        (status = 403, description = "Project visible to the caller, but insufficient rights for this operation.",
         body = String, content_type = "text/plain", example = json!("Forbidden.")),
        (status = 404, description = "Unknown project, or project not visible to the caller — both cases are deliberately indistinguishable.",
         body = String, content_type = "text/plain", example = json!("Not found.")),
        (status = 409, description = "Another status of the project already has this name.",
         body = String, content_type = "text/plain", example = json!("Another status of the project already has this name.")),
        (status = 500, description = "Database error.", body = String, content_type = "text/plain",
         example = json!("An error occurred while accessing the database.")),
    ),
    security(("jwt" = [])),
    tag = "Statuses",
)]
#[post("/")]
pub async fn create_project_status(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<ProjectPathParams>,
    view: ValidatedJson<CreateProjectStatusView>,
) -> Result<impl Responder, StatusesError> {
    let view = view.into_inner();
    let (mut tx, _) = begin_write(
        &state,
        auth_user.id,
        params.project_id(),
        None,
        Requirement::ManageProject,
    )
    .await?;
    let status_id: i32 = tx
        .fetch_scalar(&CreateProjectStatusQueryView::new(
            params.project_id(),
            view.name.trim(),
            view.color.as_deref(),
            view.category,
            view.position,
        ))
        .await
        .map_err(|e| write_error("projects/project_id/statuses/post", &e))?;
    commit(tx, "projects/project_id/statuses/post").await?;
    Ok(HttpResponse::Ok().json(CreateProjectStatusResultView {
        status_id: id_from_sql(status_id),
    }))
}

#[utoipa::path(
    patch,
    params(ProjectStatusPathParams),
    path = "",
    summary = "Update a task status of a project",
    description = "Renames, recolours, moves or recategorizes a status; an absent field is left \
                   unchanged. Reserved to the project managers. A new `position` shifts the \
                   statuses in between. A new `category` moves the tasks in the status to it (a \
                   `Completed` status archives them), and is refused (`409`) when it would leave \
                   the former category without any status. The response has an empty body.",
    request_body(content = PatchProjectStatusView, example = json!({ "name": "En validation", "position": 1 })),
    responses(
        (status = 204, description = "Status updated. Empty body."),
        (status = 400, description = "A URL segment is not an integer, malformed JSON body, `category` not one of the enum values, or a field breaking the rules of the creation.",
         body = String, content_type = "text/plain", example = json!("Invalid `name`: must not be blank")),
        (status = 401, description = "Missing `Authorization` header, invalid or expired JWT, or revoked session.",
         body = String, content_type = "text/plain", example = json!("Jeton expiré")),
        (status = 403, description = "Project visible to the caller, but insufficient rights for this operation.",
         body = String, content_type = "text/plain", example = json!("Forbidden.")),
        (status = 404, description = "Unknown or invisible project (`Not found.`), or no such status in the project (`Unknown status.`).",
         body = String, content_type = "text/plain", example = json!("Unknown status.")),
        (status = 409, description = "Another status of the project already has this name, or the new category would leave the former one without any status.",
         body = String, content_type = "text/plain", example = json!("A project keeps at least one status of each category (Todo, InProgress, Completed).")),
        (status = 500, description = "Database error.", body = String, content_type = "text/plain",
         example = json!("An error occurred while accessing the database.")),
    ),
    security(("jwt" = [])),
    tag = "Statuses",
)]
#[patch("/{status_id}/")]
pub async fn update_project_status(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<ProjectStatusPathParams>,
    view: ValidatedJson<PatchProjectStatusView>,
) -> Result<impl Responder, StatusesError> {
    let view = view.into_inner();
    let (mut tx, _) = begin_write(
        &state,
        auth_user.id,
        params.project_id(),
        None,
        Requirement::ManageProject,
    )
    .await?;
    let result: i32 = tx
        .fetch_scalar(&UpdateProjectStatusQueryView::new(
            params.project_id(),
            params.status_id(),
            StatusChanges {
                name: view.name.as_deref().map(str::trim),
                color: view.color.as_deref(),
                category: view.category,
                position: view.position,
            },
        ))
        .await
        .map_err(|e| write_error("projects/project_id/statuses/status_id/patch", &e))?;
    match result {
        STATUS_UPDATED => {}
        STATUS_LAST_OF_CATEGORY => return Err(StatusesError::LastOfCategory),
        _ => return Err(StatusesError::UnknownStatus),
    }
    commit(tx, "projects/project_id/statuses/status_id/patch").await?;
    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
    delete,
    params(ProjectStatusPathParams, DeleteProjectStatusQuery),
    path = "",
    summary = "Delete a task status of a project",
    description = "Deletes a status of the project. Reserved to the project managers. Its tasks, \
                   archived ones included, must go somewhere: `move_to` names another status of \
                   the project, and is required as soon as the status holds tasks (`409` \
                   otherwise). Each moved task takes the category of its new status and gets a \
                   `status_changed` history entry signed by the caller. The last status of a \
                   category cannot be deleted (`409`). The response has an empty body.",
    responses(
        (status = 204, description = "Status deleted. Empty body."),
        (status = 400, description = "A URL segment or `move_to` is not an integer, or `move_to` is not another status of the project.",
         body = String, content_type = "text/plain", example = json!("`move_to` is not another status of the project.")),
        (status = 401, description = "Missing `Authorization` header, invalid or expired JWT, or revoked session.",
         body = String, content_type = "text/plain", example = json!("Jeton expiré")),
        (status = 403, description = "Project visible to the caller, but insufficient rights for this operation.",
         body = String, content_type = "text/plain", example = json!("Forbidden.")),
        (status = 404, description = "Unknown or invisible project (`Not found.`), or no such status in the project (`Unknown status.`).",
         body = String, content_type = "text/plain", example = json!("Unknown status.")),
        (status = 409, description = "The status holds tasks and no `move_to` was given, or it is the last status of its category.",
         body = String, content_type = "text/plain", example = json!("The status holds tasks: give `move_to`, the status to move them to.")),
        (status = 500, description = "Database error.", body = String, content_type = "text/plain",
         example = json!("An error occurred while accessing the database.")),
    ),
    security(("jwt" = [])),
    tag = "Statuses",
)]
#[delete("/{status_id}/")]
pub async fn delete_project_status(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<ProjectStatusPathParams>,
    query: web::Query<DeleteProjectStatusQuery>,
) -> Result<impl Responder, StatusesError> {
    const OPERATION: &str = "projects/project_id/statuses/status_id/delete";
    let (project_id, status_id) = (params.project_id(), params.status_id());
    let (mut tx, _) = begin_write(
        &state,
        auth_user.id,
        project_id,
        None,
        Requirement::ManageProject,
    )
    .await?;
    let check: i32 = tx
        .fetch_scalar(&CheckStatusDeletionQueryView::new(
            project_id,
            status_id,
            query.move_to,
        ))
        .await
        .map_err(|e| write_error(OPERATION, &e))?;
    match check {
        DELETION_ALLOWED => {}
        DELETION_TASKS_TO_MOVE => return Err(StatusesError::TasksToMove),
        DELETION_INVALID_TARGET => return Err(StatusesError::InvalidTarget),
        DELETION_LAST_OF_CATEGORY => return Err(StatusesError::LastOfCategory),
        _ => return Err(StatusesError::UnknownStatus),
    }
    if let Some(move_to) = query.move_to {
        tx.execute(&MoveStatusTasksQueryView::new(
            project_id,
            status_id,
            move_to,
            auth_user.id,
        ))
        .await
        .map_err(|e| write_error(OPERATION, &e))?;
    }
    tx.execute(&DeleteProjectStatusQueryView::new(project_id, status_id))
        .await
        .map_err(|e| write_error(OPERATION, &e))?;
    commit(tx, OPERATION).await?;
    Ok(HttpResponse::NoContent().finish())
}
