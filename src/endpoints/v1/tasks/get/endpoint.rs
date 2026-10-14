use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::paged::PagedRows;
use crate::database::tasks::get_visible_tasks::view::{GetVisibleTasksQueryView, VisibleTaskRow};
use crate::endpoints::db_error::log_db_error;
use crate::endpoints::v1::tasks::get::view::{GetTasksListResultView, GetTasksQuery};
use crate::endpoints::validation::ValidatedQuery;

#[derive(Debug, Clone, PartialEq)]
pub enum GetVisibleTasksError {
    DatabaseError,
}

impl std::fmt::Display for GetVisibleTasksError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetVisibleTasksError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for GetVisibleTasksError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetVisibleTasksError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_get_tasks(
    state: web::Data<AppState>,
    user_id: u64,
    query: &GetTasksQuery,
) -> Result<GetTasksListResultView, GetVisibleTasksError> {
    let page = query.page();
    let view = GetVisibleTasksQueryView::new(user_id, &query.filters(), page.limit, page.offset);
    let result: PagedRows<VisibleTaskRow> =
        state.get_smart_db().fetch_one(&view).await.map_err(|e| {
            log_db_error("tasks/get", &e);
            GetVisibleTasksError::DatabaseError
        })?;
    Ok(result.into())
}

#[utoipa::path(
    get,
    path = "",
    summary = "List the tasks visible to me across every project",
    description = "Returns one page of the tasks of every project visible to the user of the JWT \
                   (the projects of `GET /api/v1/projects/`: Admin and Maire see every project, a \
                   Responsable also the projects of their groups' members, anyone else the projects \
                   they own or are a member of), each with the id and name of its project. One \
                   call gives a dashboard the pending tasks of a user (`assigned_to` = their id, \
                   optionally `due_before`) instead of reading the detail of each project \
                   (MAIR-524).\n\n\
                   Tasks are ordered by due date, soonest first (tasks without due date last), then \
                   by id. Without `status`, only the active tasks are listed: a `Completed` task is \
                   archived (MAIR-502) and only listed when `status` includes `Completed`. The \
                   filters (`assigned_to`, `status`, `due_before`) are optional and combine. \
                   `limit` (default 100, at most 500) and `offset` (default 0) select the page; \
                   `total` counts every matching task, whatever the page. The list is empty (never \
                   `404`) when no visible task matches, including for an `assigned_to` the caller \
                   shares no project with. Not cached: a write is visible on the next read.",
    params(
        GetTasksQuery
    ),
    responses(
        (
            status = 200,
            description = "One page of the tasks visible to the caller.",
            body = GetTasksListResultView,
            example = json!({
                "tasks": [
                    {
                        "id": 87, "title": "Consulter les riverains",
                        "description": "Réunion publique à organiser avant le 15 octobre",
                        "status": "InProgress", "priority": "High",
                        "due_date": "2026-10-15T00:00:00Z", "archived_at": null,
                        "assigned_to": 42, "fields": [],
                        "project_id": 12, "project_name": "Réfection de la place du marché"
                    },
                    {
                        "id": 93, "title": "Numériser les registres de 1950", "description": "",
                        "status": "Todo", "priority": "Medium",
                        "due_date": null, "archived_at": null,
                        "assigned_to": 42, "fields": [],
                        "project_id": 18, "project_name": "Numérisation de l'état civil"
                    }
                ],
                "total": 2
            })
        ),
        (
            status = 400,
            description = "`limit`, `offset` or `assigned_to` is not a non-negative integer, \
                           `due_before` is not an RFC 3339 instant, or `status` holds a value outside \
                           `Todo`, `InProgress`, `Completed`, `Error`. The body names the first \
                           invalid parameter.",
            body = String,
            content_type = "text/plain",
            example = json!("Invalid `status`: must be a comma-separated list of Todo, InProgress, Completed, Error")
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
            description = "The account of the JWT does not exist or is archived.",
            body = String,
            content_type = "text/plain",
            example = json!("Utilisateur inconnu")
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
pub async fn get_tasks(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    query: ValidatedQuery<GetTasksQuery>,
) -> Result<impl Responder, GetVisibleTasksError> {
    let result = trigger_get_tasks(state, auth_user.id, &query.into_inner()).await?;
    Ok(HttpResponse::Ok().json(result))
}
