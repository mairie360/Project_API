use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::activity::view::{ActivityRow, GetProjectActivityQueryView};
use crate::database::paged::PagedRows;
use crate::endpoints::db_error::log_db_error;
use crate::endpoints::pagination::{Page, PageParams};
use crate::endpoints::v1::projects::access::{require_access, AccessDenied, Requirement};
use crate::endpoints::v1::projects::project_id::activity::view::GetProjectActivityResultView;
use crate::endpoints::v1::projects::project_id::ProjectPathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum GetProjectActivityError {
    NotFound,
    DatabaseError,
}

impl std::fmt::Display for GetProjectActivityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetProjectActivityError::NotFound => write!(f, "Not found."),
            GetProjectActivityError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for GetProjectActivityError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetProjectActivityError::NotFound => StatusCode::NOT_FOUND,
            GetProjectActivityError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

impl From<AccessDenied> for GetProjectActivityError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            // A read only needs to see the project: no `Forbidden` is possible.
            AccessDenied::NotFound | AccessDenied::Forbidden => GetProjectActivityError::NotFound,
            AccessDenied::DatabaseError => GetProjectActivityError::DatabaseError,
        }
    }
}

async fn trigger_get_project_activity(
    state: &web::Data<AppState>,
    project_id: u64,
    page: Page,
) -> Result<GetProjectActivityResultView, GetProjectActivityError> {
    let rows: PagedRows<ActivityRow> = state
        .get_smart_db()
        .fetch_one(&GetProjectActivityQueryView::new(
            project_id,
            page.limit,
            page.offset,
        ))
        .await
        .map_err(|e| {
            log_db_error("projects/project_id/activity/get", &e);
            GetProjectActivityError::DatabaseError
        })?;
    Ok(GetProjectActivityResultView {
        items: rows.items.into_iter().map(Into::into).collect(),
        total: u64::try_from(rows.total).unwrap_or(0),
    })
}

#[utoipa::path(
    get,
    params(ProjectPathParams, PageParams),
    path = "",
    summary = "Read the activity of a project",
    description = "Returns one page of the activity of the project (MAIR-1089), newest first: its \
                   comments (`POST …/comments/`) and the history entries of all its tasks, \
                   archived ones included (`task_created`, `task_updated`, `status_changed`, \
                   written by the database). `limit` (default 100, at most 500) and `offset` \
                   (default 0) page it, `total` counts every entry. Being able to see the project \
                   is enough.",
    responses(
        (status = 200, description = "One page of the activity of the project.", body = GetProjectActivityResultView,
         example = json!({ "items": [
             { "kind": "comment", "id": 31, "task_id": null, "task_title": null, "author_id": 42,
               "author_name": "Jean Dupont", "message": "The tender closes on Friday.", "changes": null,
               "label": "Comment", "created_at": "2026-11-21T09:30:00.000Z" },
             { "kind": "status_changed", "id": 8, "task_id": 77, "task_title": "Consult the residents",
               "author_id": 51, "author_name": "Amina Bensaïd", "message": null,
               "changes": { "status": { "from": "todo", "to": "in_progress" } },
               "label": "Status changed: À faire → En cours", "created_at": "2026-11-20T15:04:00.000Z" }
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
    tag = "Activity",
)]
#[get("/")]
pub async fn get_project_activity(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<ProjectPathParams>,
    page: web::Query<PageParams>,
) -> Result<impl Responder, GetProjectActivityError> {
    require_access(
        &state,
        auth_user.id,
        params.project_id(),
        None,
        Requirement::ViewProject,
    )
    .await?;
    let result = trigger_get_project_activity(&state, params.project_id(), page.page()).await?;
    Ok(HttpResponse::Ok().json(result))
}
