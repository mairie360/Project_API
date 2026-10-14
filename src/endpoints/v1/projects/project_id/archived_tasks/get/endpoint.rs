use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::endpoints::db_error::log_db_error;

use crate::database::paged::PagedRows;
use crate::database::tasks::get_project_tasks::view::{GetProjectTasksQueryView, Task};
use crate::endpoints::pagination::{Page, PageParams};
use crate::endpoints::v1::projects::access::{require_access, AccessDenied, Requirement};
use crate::endpoints::v1::projects::project_id::tasks::get::view::GetTasksResultView;
use crate::endpoints::v1::projects::project_id::ProjectPathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum GetArchivedTasksError {
    Forbidden,
    NotFound,
    BadRequest,
    DatabaseError,
}

impl std::fmt::Display for GetArchivedTasksError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetArchivedTasksError::Forbidden => write!(f, "Forbidden."),
            GetArchivedTasksError::NotFound => write!(f, "Not found."),
            GetArchivedTasksError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            GetArchivedTasksError::BadRequest => {
                write!(f, "Bad request.")
            }
        }
    }
}

impl ResponseError for GetArchivedTasksError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetArchivedTasksError::Forbidden => StatusCode::FORBIDDEN,
            GetArchivedTasksError::NotFound => StatusCode::NOT_FOUND,
            GetArchivedTasksError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            GetArchivedTasksError::BadRequest => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_get_archived_tasks(
    state: web::Data<AppState>,
    project_id: u64,
    page: Page,
) -> Result<GetTasksResultView, GetArchivedTasksError> {
    let view = GetProjectTasksQueryView::archived(project_id, page.limit, page.offset);
    let result: PagedRows<Task> = state.get_smart_db().fetch_one(&view).await.map_err(|e| {
        log_db_error("projects/project_id/archived_tasks/get", &e);
        GetArchivedTasksError::DatabaseError
    })?;

    Ok(GetTasksResultView {
        tasks: result.items.into_iter().map(Into::into).collect(),
        total: u64::try_from(result.total).unwrap_or(0),
    })
}

#[utoipa::path(
    get,
    params(
        ProjectPathParams,
        PageParams,
    ),
    path = "",
    summary = "List the archived tasks of a project",
    description = "Returns one page of the archived tasks of the project (MAIR-502): a task is \
                   archived as soon as it is `Completed`, and comes back to the active tasks of \
                   `GET /api/v1/projects/{project_id}/tasks/` when it is reopened. The most recently \
                   archived first, each with its `archived_at`. Being a member of the project is \
                   enough.\n\n\
                   `limit` (default 100, at most 500) and `offset` (default 0) select the page; \
                   `total` is the number of archived tasks of the project, whatever the page.",
    responses(
        (
            status = 200,
            description = "One page of the archived tasks of the project. Empty if it has none, or if `offset` is past the end.",
            body = GetTasksResultView,
            example = json!({
                "tasks": [
                    {
                        "id": 77,
                        "title": "Consulter les riverains",
                        "description": "Réunion publique à organiser avant le 15 octobre",
                        "status": "Completed",
                        "priority": "High",
                        "due_date": "2026-10-15T00:00:00Z",
                        "archived_at": "2026-10-02T14:30:00Z",
                        "assigned_to": 42,
                        "fields": [
                            { "label": "Date de la réunion", "task_type": "date", "fields_options": [] }
                        ]
                    }
                ],
                "total": 1
            })
        ),
        (
            status = 400,
            description = "A URL segment is not an integer, or `limit` / `offset` is not a non-negative integer.",
            body = String,
            content_type = "text/plain",
            example = json!("Path deserialize error: can not parse `abc` to a u64")
        ),
        (
            status = 401,
            description = "Missing `Authorization` header, invalid or expired JWT, or revoked session.",
            body = String,
            content_type = "text/plain",
            example = json!("Jeton expiré")
        ),
        (
            status = 404,
            description = "Unknown project, or project not visible to the caller — both cases are deliberately indistinguishable.",
            body = String,
            content_type = "text/plain",
            example = json!("Not found.")
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
    tag = "Tasks",
)]
#[get("/")]
pub async fn get_archived_tasks(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<ProjectPathParams>,
    page: web::Query<PageParams>,
) -> Result<impl Responder, GetArchivedTasksError> {
    require_access(
        &state,
        auth_user.id,
        params.project_id(),
        None,
        Requirement::ViewProject,
    )
    .await?;
    let result = trigger_get_archived_tasks(state, params.project_id, page.page()).await?;
    Ok(HttpResponse::Ok().json(result))
}

impl From<AccessDenied> for GetArchivedTasksError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => GetArchivedTasksError::NotFound,
            AccessDenied::Forbidden => GetArchivedTasksError::Forbidden,
            AccessDenied::DatabaseError => GetArchivedTasksError::DatabaseError,
        }
    }
}
