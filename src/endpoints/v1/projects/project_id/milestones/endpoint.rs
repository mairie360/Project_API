use actix_web::{delete, get, patch, post, web, HttpResponse, Responder};
use mairie360_api_lib::error::ApiLibError;
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::milestones::view::{
    CreateMilestoneQueryView, DeleteMilestoneQueryView, GetProjectMilestonesQueryView,
    MilestoneChanges, MilestoneRow, PatchMilestoneQueryView,
};
use crate::database::paged::PagedRows;
use crate::endpoints::db_error::{classify_db_error, log_db_error, DbFailure};
use crate::endpoints::pagination::PageParams;
use crate::endpoints::v1::projects::access::{begin_write, commit, require_access, Requirement};
use crate::endpoints::v1::projects::project_id::milestones::view::{
    CreateMilestoneView, GetProjectMilestonesResultView, MilestoneView, PatchMilestoneView,
};
use crate::endpoints::v1::projects::project_id::milestones::{
    MilestonesError, ProjectMilestonePathParams,
};
use crate::endpoints::v1::projects::project_id::ProjectPathParams;
use crate::endpoints::validation::ValidatedJson;

/// A check constraint (`23514`, e.g. a title the database refuses) is the client's fault; anything
/// else is logged and becomes a `500`.
fn write_error(operation: &str, error: &ApiLibError) -> MilestonesError {
    match classify_db_error(operation, error) {
        DbFailure::CheckViolation => MilestonesError::BadRequest,
        _ => MilestonesError::DatabaseError,
    }
}

#[utoipa::path(
    get,
    params(ProjectPathParams, PageParams),
    path = "",
    summary = "List the milestones of a project",
    description = "Returns one page of the milestones of the project (MAIR-1086), by due date then \
                   id; deleted milestones are left out. `limit` (default 100, at most 500) and \
                   `offset` (default 0) page them, `total` counts them all. Being able to see the \
                   project is enough. `GET /api/v1/projects/{project_id}/` also embeds the first \
                   100.",
    responses(
        (status = 200, description = "One page of the milestones of the project.", body = GetProjectMilestonesResultView,
         example = json!({ "milestones": [
             { "id": 5, "project_id": 12, "title": "Tender awarded", "due_date": "2026-12-15", "reached": true,
               "created_by": 42, "updated_by": 42, "created_at": "2026-11-21T09:30:00Z", "updated_at": "2026-12-16T08:00:00Z" },
             { "id": 6, "project_id": 12, "title": "Works delivered", "due_date": "2027-06-30", "reached": false,
               "created_by": 42, "updated_by": 42, "created_at": "2026-11-21T09:31:00Z", "updated_at": "2026-11-21T09:31:00Z" }
         ], "total": 2 })),
        (status = 400, description = "A URL segment is not an integer, or `limit` / `offset` is not a non-negative integer.",
         body = String, content_type = "text/plain", example = json!("Path deserialize error: can not parse `abc` to a u64")),
        (status = 401, description = "Missing `Authorization` header, invalid or expired JWT, or revoked session.",
         body = String, content_type = "text/plain", example = json!("Jeton expiré")),
        (status = 404, description = "Unknown project, or project not visible to the caller — both cases are deliberately indistinguishable.",
         body = String, content_type = "text/plain", example = json!("Not found.")),
        (status = 500, description = "Database error.", body = String, content_type = "text/plain",
         example = json!("An error occurred while accessing the database.")),
    ),
    security(("jwt" = [])),
    tag = "Milestones",
)]
#[get("/")]
pub async fn get_project_milestones(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<ProjectPathParams>,
    page: web::Query<PageParams>,
) -> Result<impl Responder, MilestonesError> {
    require_access(
        &state,
        auth_user.id,
        params.project_id(),
        None,
        Requirement::ViewProject,
    )
    .await?;
    let page = page.page();
    let rows: PagedRows<MilestoneRow> = state
        .get_smart_db()
        .fetch_one(&GetProjectMilestonesQueryView::new(
            params.project_id(),
            page.limit,
            page.offset,
        ))
        .await
        .map_err(|e| {
            log_db_error("projects/project_id/milestones/get", &e);
            MilestonesError::DatabaseError
        })?;
    Ok(HttpResponse::Ok().json(GetProjectMilestonesResultView {
        milestones: rows.items.into_iter().map(Into::into).collect(),
        total: u64::try_from(rows.total).unwrap_or(0),
    }))
}

#[utoipa::path(
    post,
    params(ProjectPathParams),
    path = "",
    summary = "Add a milestone to a project",
    description = "Adds a dated milestone to the project (MAIR-1086), signed by the caller. \
                   Reserved to the project managers. Answers the milestone created.",
    request_body(content = CreateMilestoneView, example = json!({
        "title": "Tender awarded", "due_date": "2026-12-15", "reached": false
    })),
    responses(
        (status = 201, description = "Milestone added.", body = MilestoneView,
         example = json!({ "id": 5, "project_id": 12, "title": "Tender awarded", "due_date": "2026-12-15", "reached": false,
                           "created_by": 42, "updated_by": 42, "created_at": "2026-11-21T09:30:00Z", "updated_at": "2026-11-21T09:30:00Z" })),
        (status = 400, description = "A URL segment is not an integer, malformed JSON body, `due_date` not a `YYYY-MM-DD` day, or `title` breaking its rules: 1 to 255 characters, not blank, no control character.",
         body = String, content_type = "text/plain", example = json!("Invalid `title`: must not be empty")),
        (status = 401, description = "Missing `Authorization` header, invalid or expired JWT, or revoked session.",
         body = String, content_type = "text/plain", example = json!("Jeton expiré")),
        (status = 403, description = "Project visible to the caller, but insufficient rights for this operation.",
         body = String, content_type = "text/plain", example = json!("Forbidden.")),
        (status = 404, description = "Unknown project, or project not visible to the caller — both cases are deliberately indistinguishable.",
         body = String, content_type = "text/plain", example = json!("Not found.")),
        (status = 409, description = "The caller may write, but the project is closed (`Completed`): it is read-only until a manager reopens it.",
         body = String, content_type = "text/plain", example = json!("Project is closed.")),
        (status = 500, description = "Database error.", body = String, content_type = "text/plain",
         example = json!("An error occurred while accessing the database.")),
    ),
    security(("jwt" = [])),
    tag = "Milestones",
)]
#[post("/")]
pub async fn create_project_milestone(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<ProjectPathParams>,
    view: ValidatedJson<CreateMilestoneView>,
) -> Result<impl Responder, MilestonesError> {
    const OPERATION: &str = "projects/project_id/milestones/post";
    let view = view.into_inner();
    let (mut tx, _) = begin_write(
        &state,
        auth_user.id,
        params.project_id(),
        None,
        Requirement::ManageProject,
    )
    .await?;
    let row: MilestoneRow = tx
        .fetch_one(&CreateMilestoneQueryView::new(
            params.project_id(),
            auth_user.id,
            view.title.trim(),
            view.due_date,
            view.reached,
        ))
        .await
        .map_err(|e| write_error(OPERATION, &e))?;
    commit(tx, OPERATION).await?;
    Ok(HttpResponse::Created().json(MilestoneView::from(row)))
}

#[utoipa::path(
    patch,
    params(ProjectMilestonePathParams),
    path = "",
    summary = "Update a milestone of a project",
    description = "Renames, reschedules, marks reached or reopens a milestone (MAIR-1086); an \
                   absent or `null` field is kept. Reserved to the project managers. A milestone \
                   of another project, or a deleted one, answers `404`. Answers the milestone as \
                   updated.",
    request_body(content = PatchMilestoneView, example = json!({ "reached": true })),
    responses(
        (status = 200, description = "Milestone updated.", body = MilestoneView,
         example = json!({ "id": 5, "project_id": 12, "title": "Tender awarded", "due_date": "2026-12-15", "reached": true,
                           "created_by": 42, "updated_by": 51, "created_at": "2026-11-21T09:30:00Z", "updated_at": "2026-12-16T08:00:00Z" })),
        (status = 400, description = "A URL segment is not an integer, malformed JSON body, `due_date` not a `YYYY-MM-DD` day, or `title` breaking the rules of the creation.",
         body = String, content_type = "text/plain", example = json!("Invalid `title`: must not be empty")),
        (status = 401, description = "Missing `Authorization` header, invalid or expired JWT, or revoked session.",
         body = String, content_type = "text/plain", example = json!("Jeton expiré")),
        (status = 403, description = "Project visible to the caller, but insufficient rights for this operation.",
         body = String, content_type = "text/plain", example = json!("Forbidden.")),
        (status = 404, description = "Unknown or invisible project (`Not found.`), or no such milestone in the project, or a deleted one (`Unknown milestone.`).",
         body = String, content_type = "text/plain", example = json!("Unknown milestone.")),
        (status = 409, description = "The caller may write, but the project is closed (`Completed`): it is read-only until a manager reopens it.",
         body = String, content_type = "text/plain", example = json!("Project is closed.")),
        (status = 500, description = "Database error.", body = String, content_type = "text/plain",
         example = json!("An error occurred while accessing the database.")),
    ),
    security(("jwt" = [])),
    tag = "Milestones",
)]
#[patch("/{milestone_id}/")]
pub async fn update_project_milestone(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<ProjectMilestonePathParams>,
    view: ValidatedJson<PatchMilestoneView>,
) -> Result<impl Responder, MilestonesError> {
    const OPERATION: &str = "projects/project_id/milestones/milestone_id/patch";
    let view = view.into_inner();
    let (mut tx, _) = begin_write(
        &state,
        auth_user.id,
        params.project_id(),
        None,
        Requirement::ManageProject,
    )
    .await?;
    let rows: Vec<MilestoneRow> = tx
        .fetch_all(&PatchMilestoneQueryView::new(
            params.project_id(),
            params.milestone_id(),
            auth_user.id,
            &MilestoneChanges {
                title: view.title.as_deref().map(str::trim),
                due_date: view.due_date,
                reached: view.reached,
            },
        ))
        .await
        .map_err(|e| write_error(OPERATION, &e))?;
    let row = rows
        .into_iter()
        .next()
        .ok_or(MilestonesError::UnknownMilestone)?;
    commit(tx, OPERATION).await?;
    Ok(HttpResponse::Ok().json(MilestoneView::from(row)))
}

#[utoipa::path(
    delete,
    params(ProjectMilestonePathParams),
    path = "",
    summary = "Delete a milestone of a project",
    description = "Deletes a milestone of the project (MAIR-1086): it is kept in the database \
                   (`deleted_at`) but no longer read. Reserved to the project managers. A \
                   milestone of another project, or one already deleted, answers `404`. The \
                   response has an empty body.",
    responses(
        (status = 204, description = "Milestone deleted. Empty body."),
        (status = 400, description = "A URL segment is not an integer.", body = String, content_type = "text/plain",
         example = json!("Path deserialize error: can not parse `abc` to a u64")),
        (status = 401, description = "Missing `Authorization` header, invalid or expired JWT, or revoked session.",
         body = String, content_type = "text/plain", example = json!("Jeton expiré")),
        (status = 403, description = "Project visible to the caller, but insufficient rights for this operation.",
         body = String, content_type = "text/plain", example = json!("Forbidden.")),
        (status = 404, description = "Unknown or invisible project (`Not found.`), or no such milestone in the project, or one already deleted (`Unknown milestone.`).",
         body = String, content_type = "text/plain", example = json!("Unknown milestone.")),
        (status = 409, description = "The caller may write, but the project is closed (`Completed`): it is read-only until a manager reopens it.",
         body = String, content_type = "text/plain", example = json!("Project is closed.")),
        (status = 500, description = "Database error.", body = String, content_type = "text/plain",
         example = json!("An error occurred while accessing the database.")),
    ),
    security(("jwt" = [])),
    tag = "Milestones",
)]
#[delete("/{milestone_id}/")]
pub async fn delete_project_milestone(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<ProjectMilestonePathParams>,
) -> Result<impl Responder, MilestonesError> {
    const OPERATION: &str = "projects/project_id/milestones/milestone_id/delete";
    let (mut tx, _) = begin_write(
        &state,
        auth_user.id,
        params.project_id(),
        None,
        Requirement::ManageProject,
    )
    .await?;
    let deleted: bool = tx
        .fetch_scalar(&DeleteMilestoneQueryView::new(
            params.project_id(),
            params.milestone_id(),
            auth_user.id,
        ))
        .await
        .map_err(|e| write_error(OPERATION, &e))?;
    if !deleted {
        return Err(MilestonesError::UnknownMilestone);
    }
    commit(tx, OPERATION).await?;
    Ok(HttpResponse::NoContent().finish())
}
