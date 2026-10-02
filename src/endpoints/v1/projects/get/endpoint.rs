use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::paged::PagedRows;
use crate::endpoints::db_error::log_db_error;
use crate::endpoints::pagination::{Page, PageParams};

use crate::database::project::get_projects::view::{GetProjectsQueryView, ProjectView};
use crate::endpoints::v1::projects::get::view::GetProjectsResultView;

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
    page: Page,
) -> Result<GetProjectsResultView, GetProjectsError> {
    let view = GetProjectsQueryView::new(user_id, page.limit, page.offset);
    let result: PagedRows<ProjectView> =
        state.get_smart_db().fetch_one(&view).await.map_err(|e| {
            log_db_error("projects/get", &e);
            GetProjectsError::DatabaseError
        })?;

    Ok(GetProjectsResultView {
        projects: result.items.into_iter().map(Into::into).collect(),
        total: result.total.max(0) as u64,
    })
}

#[utoipa::path(
    get,
    path = "",
    summary = "List my projects",
    description = "Returns one page of the projects visible to the user of the JWT, newest first. \
                   List view: neither tasks nor members are included, call \
                   `GET /api/v1/projects/{project_id}/` for the detail of a project.\n\n\
                   `limit` (default 100, at most 500) and `offset` (default 0) select the page; \
                   `total` is the number of visible projects, whatever the page. The list is \
                   empty if the user has access to no project.",
    params(
        PageParams
    ),
    responses(
        (
            status = 200,
            description = "One page of the projects visible to the caller.",
            body = GetProjectsResultView,
            example = json!({
                "projects": [
                    { "id": 12, "name": "Réfection de la place du marché", "description": "Travaux de voirie 2026", "status": "Active" },
                    { "id": 18, "name": "Numérisation de l'état civil", "description": "", "status": "Completed" }
                ],
                "total": 2
            })
        ),
        (
            status = 400,
            description = "`limit` or `offset` is not a non-negative integer.",
            body = String,
            content_type = "text/plain",
            example = json!("Query deserialize error: invalid digit found in string")
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
    page: web::Query<PageParams>,
) -> Result<impl Responder, GetProjectsError> {
    let result = trigger_get_projects(state, auth_user.id, page.page()).await?;
    Ok(HttpResponse::Ok().json(result))
}
