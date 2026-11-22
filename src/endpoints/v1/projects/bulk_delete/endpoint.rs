use actix_web::http::StatusCode;
use actix_web::{post, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::project::delete::view::DeleteProjectQueryView;
use crate::endpoints::db_error::log_db_error;
use crate::endpoints::v1::projects::access::{begin_write, commit, AccessDenied, Requirement};
use crate::endpoints::v1::projects::bulk_delete::view::{
    BulkDeleteProjectsResultView, BulkDeleteProjectsView, BulkDeleteResultItem, BulkDeleteStatus,
};
use crate::endpoints::validation::ValidatedJson;

#[derive(Debug, Clone, PartialEq)]
pub enum BulkDeleteProjectsError {
    DatabaseError,
}

impl std::fmt::Display for BulkDeleteProjectsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BulkDeleteProjectsError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for BulkDeleteProjectsError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

/// Deletes one project in a transaction of its own, after the access check of
/// `DELETE /projects/{project_id}/` (`begin_write`, the project row locked).
async fn delete_one(
    state: &web::Data<AppState>,
    user_id: u64,
    project_id: u64,
) -> Result<BulkDeleteStatus, BulkDeleteProjectsError> {
    const OPERATION: &str = "projects/bulk-delete";
    let mut tx =
        match begin_write(state, user_id, project_id, None, Requirement::ManageProject).await {
            Ok((tx, _)) => tx,
            Err(AccessDenied::NotFound) => return Ok(BulkDeleteStatus::NotFound),
            Err(AccessDenied::Forbidden) => return Ok(BulkDeleteStatus::Forbidden),
            Err(AccessDenied::DatabaseError) => return Err(BulkDeleteProjectsError::DatabaseError),
        };
    tx.execute(&DeleteProjectQueryView::new(project_id))
        .await
        .map_err(|e| {
            log_db_error(OPERATION, &e);
            BulkDeleteProjectsError::DatabaseError
        })?;
    commit(tx, OPERATION)
        .await
        .map_err(|_| BulkDeleteProjectsError::DatabaseError)?;
    Ok(BulkDeleteStatus::Deleted)
}

#[utoipa::path(
    post,
    path = "",
    summary = "Delete several projects",
    description = "Deletes up to 100 projects at once (MAIR-1089). Each project is checked and \
                   deleted on its own, exactly like `DELETE /api/v1/projects/{project_id}/`, in a \
                   transaction of its own: a project the caller may not delete never blocks the \
                   others. The answer lists, for each distinct id in the order given, `deleted`, \
                   `not_found` (unknown, or not visible to the caller) or `forbidden` (visible, \
                   but the caller is not one of its managers). A database error stops the batch \
                   with a `500`; the projects already deleted stay deleted.",
    request_body(
        content = BulkDeleteProjectsView,
        description = "Ids of the projects to delete.",
        example = json!({ "ids": [12, 18, 25] })
    ),
    responses(
        (status = 200, description = "One outcome per distinct id.", body = BulkDeleteProjectsResultView,
         example = json!({ "results": [
             { "id": 12, "status": "deleted" },
             { "id": 18, "status": "forbidden" },
             { "id": 25, "status": "not_found" }
         ] })),
        (status = 400, description = "Malformed JSON body, `ids` missing, empty, longer than 100, or holding something other than non-negative integers.",
         body = String, content_type = "text/plain", example = json!("Invalid `ids`: must hold 1 to 100 ids")),
        (status = 401, description = "Missing `Authorization` header, invalid or expired JWT, or revoked session.",
         body = String, content_type = "text/plain", example = json!("Jeton expiré")),
        (status = 500, description = "Database error.", body = String, content_type = "text/plain",
         example = json!("An error occurred while accessing the database.")),
    ),
    security(("jwt" = [])),
    tag = "Projects",
)]
#[post("/")]
pub async fn bulk_delete_projects(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    view: ValidatedJson<BulkDeleteProjectsView>,
) -> Result<impl Responder, BulkDeleteProjectsError> {
    let mut results = Vec::new();
    // One after the other: each deletion locks its project, and a failure stops the batch.
    for id in view.into_inner().unique_ids() {
        let status = delete_one(&state, auth_user.id, id).await?;
        results.push(BulkDeleteResultItem { id, status });
    }
    Ok(HttpResponse::Ok().json(BulkDeleteProjectsResultView { results }))
}
