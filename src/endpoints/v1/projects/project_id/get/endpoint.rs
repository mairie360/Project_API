use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::paged::PagedRows;
use crate::endpoints::db_error::log_db_error;
use crate::endpoints::pagination::{Page, PageParams};

use crate::database::project::get_project::view::GetVisibleProjectQueryView;
use crate::database::project::get_projects::view::ProjectView;
use crate::database::tasks::get_project_tasks::view::{GetProjectTasksQueryView, Task};
use crate::database::users::get_project_users::view::{GetProjectUsersQueryView, ProjectMemberRow};
use crate::endpoints::v1::projects::project_id::get::view::GetProjectResultView;
use crate::endpoints::v1::projects::project_id::ProjectPathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum GetProjectError {
    NotFound,
    DatabaseError,
}

impl std::fmt::Display for GetProjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetProjectError::NotFound => write!(f, "Unknown project."),
            GetProjectError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for GetProjectError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetProjectError::NotFound => StatusCode::NOT_FOUND,
            GetProjectError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_get_project(
    state: web::Data<AppState>,
    user_id: u64,
    project_id: u64,
    page: Page,
) -> Result<GetProjectResultView, GetProjectError> {
    let smart_db = state.get_smart_db();
    let projects: Vec<ProjectView> = smart_db
        .fetch_all(&GetVisibleProjectQueryView::new(project_id, user_id))
        .await
        .map_err(|e| {
            log_db_error("projects/project_id/get", &e);
            GetProjectError::DatabaseError
        })?;
    // A project the caller cannot see is treated as missing.
    let project = projects
        .into_iter()
        .next()
        .ok_or(GetProjectError::NotFound)?;

    let tasks_view = GetProjectTasksQueryView::new(project_id, page.limit, page.offset);
    let users_view = GetProjectUsersQueryView::new(project_id);
    let (tasks, users) = futures_util::try_join!(
        smart_db.fetch_one::<PagedRows<Task>, _>(&tasks_view),
        smart_db.fetch_all::<ProjectMemberRow, _>(&users_view),
    )
    .map_err(|e| {
        log_db_error("projects/project_id/get", &e);
        GetProjectError::DatabaseError
    })?;

    Ok(GetProjectResultView {
        project: project.into(),
        tasks: tasks.items.into_iter().map(Into::into).collect(),
        tasks_total: tasks.total.max(0) as u64,
        users: users.into_iter().map(Into::into).collect(),
    })
}

#[utoipa::path(
    get,
    params(
        ProjectPathParams,
        PageParams,
    ),
    path = "",
    summary = "Read a project",
    description = "Returns a project with **one page of its tasks and all its members** in a \
                   single request: the call the front makes to display a project board, rather \
                   than chaining `/tasks/` and `/users/`.\n\n\
                   `limit` (default 100, at most 500) and `offset` (default 0) page the tasks, \
                   oldest first; `tasks_total` is the number of tasks of the project. Use \
                   `GET …/tasks/` to fetch the next pages alone.\n\n\
                   Being a member of the project is enough. A project the caller cannot access \
                   answers `404`, not `403`.",
    responses(
        (
            status = 200,
            description = "The project, one page of its tasks and its members.",
            body = GetProjectResultView,
            example = json!({
                "project": { "id": 12, "name": "Réfection de la place du marché", "description": "Travaux de voirie 2026", "status": "Active" },
                "tasks": [
                    {
                        "id": 77,
                        "title": "Consulter les riverains",
                        "description": "Réunion publique à organiser avant le 15 octobre",
                        "status": "InProgress",
                        "priority": "High",
                        "due_date": "2026-10-15T00:00:00Z",
                        "assigned_to": 42,
                        "fields": []
                    }
                ],
                "tasks_total": 1,
                "users": [
                    { "id": 42, "name": "Jean Dupont" },
                    { "id": 51, "name": "Amina Bensaïd" }
                ]
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
    tag = "Projects",
)]
#[get("/")]
pub async fn get_project(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<ProjectPathParams>,
    page: web::Query<PageParams>,
) -> Result<impl Responder, GetProjectError> {
    let result = trigger_get_project(state, auth_user.id, params.project_id(), page.page()).await?;
    Ok(HttpResponse::Ok().json(result))
}
