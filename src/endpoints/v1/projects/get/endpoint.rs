use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::endpoints::db_error::log_db_error;
use crate::endpoints::validation::ValidatedQuery;

use crate::database::project::get_projects::view::{GetProjectsQueryView, ProjectsPage};
use crate::endpoints::v1::projects::get::view::{GetProjectsQuery, GetProjectsResultView};

#[derive(Debug, Clone, PartialEq)]
pub enum GetProjectsError {
    BadRequest,
    DatabaseError,
}

impl std::fmt::Display for GetProjectsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetProjectsError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            GetProjectsError::BadRequest => {
                write!(f, "Bad request.")
            }
        }
    }
}

impl ResponseError for GetProjectsError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetProjectsError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            GetProjectsError::BadRequest => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_get_projects(
    state: web::Data<AppState>,
    user_id: u64,
    query: GetProjectsQuery,
) -> Result<GetProjectsResultView, GetProjectsError> {
    let page = query.page();
    let view = GetProjectsQueryView::filtered(user_id, &query.filters(), page.limit, page.offset);
    let result: ProjectsPage = state.get_smart_db().fetch_one(&view).await.map_err(|e| {
        log_db_error("projects/get", &e);
        GetProjectsError::DatabaseError
    })?;

    Ok(result.into())
}

#[utoipa::path(
    get,
    path = "",
    summary = "List my projects",
    description = "Returns one page of the projects visible to the user of the JWT that match the \
                   filters, newest first. Each project carries the aggregates of its tasks \
                   (`tasks_total`, `tasks_completed`, `priority`: the highest priority of its \
                   tasks, `Medium` without task; `due_date`: their earliest due date) and its first \
                   5 members; call `GET /api/v1/projects/{project_id}/` for its tasks.\n\n\
                   The filters (`search`, `status`, `priority`, `due_before`, `due_after`) are \
                   optional and combine. `limit` (default 100, at most 500) and `offset` (default 0) \
                   select the page; `total` and `summary` (counts per status and per priority) cover \
                   every matching project, whatever the page. The list is empty if no visible \
                   project matches.",
    params(
        GetProjectsQuery
    ),
    responses(
        (
            status = 200,
            description = "One page of the projects visible to the caller.",
            body = GetProjectsResultView,
            example = json!({
                "projects": [
                    {
                        "id": 12, "name": "Réfection de la place du marché", "description": "Travaux de voirie 2026",
                        "status": "Active", "tasks_total": 12, "tasks_completed": 4, "priority": "High",
                        "due_date": "2026-10-15T00:00:00Z",
                        "members": [{ "id": 42, "name": "Jean Dupont" }, { "id": 57, "name": "Marie Durand" }],
                        "members_total": 2
                    },
                    {
                        "id": 18, "name": "Numérisation de l'état civil", "description": "", "status": "Completed",
                        "tasks_total": 0, "tasks_completed": 0, "priority": "Medium", "due_date": null,
                        "members": [], "members_total": 0
                    }
                ],
                "total": 2,
                "summary": {
                    "by_status": { "active": 1, "suspended": 0, "completed": 1, "other": 0 },
                    "by_priority": { "low": 0, "medium": 1, "high": 1 }
                }
            })
        ),
        (
            status = 400,
            description = "`limit` or `offset` is not a non-negative integer, `due_before` or `due_after` \
                           is not an RFC 3339 instant, `status` or `priority` holds a value outside \
                           its list, or `search` is longer than 255 characters or holds a control \
                           character. The body names the first invalid parameter.",
            body = String,
            content_type = "text/plain",
            example = json!("Invalid `priority`: must be a comma-separated list of Low, Medium, High")
        ),
        (
            status = 401,
            description = "Missing `Authorization` header, invalid or expired JWT, or revoked session.",
            body = String,
            content_type = "text/plain",
            example = json!("Jeton expiré")
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
pub async fn get_projects(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    query: ValidatedQuery<GetProjectsQuery>,
) -> Result<impl Responder, GetProjectsError> {
    let result = trigger_get_projects(state, auth_user.id, query.into_inner()).await?;
    Ok(HttpResponse::Ok().json(result))
}
