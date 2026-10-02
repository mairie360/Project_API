use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::paged::PagedRows;
use crate::endpoints::db_error::log_db_error;
use crate::endpoints::pagination::{Page, PageParams};

use crate::database::users::get_project_users::view::{GetProjectUsersQueryView, ProjectMemberRow};
use crate::endpoints::v1::projects::access::{require_access, AccessDenied, Requirement};
use crate::endpoints::v1::projects::project_id::users::get::view::{
    GetProjectUsersResultView, User,
};
use crate::endpoints::v1::projects::project_id::ProjectPathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum GetProjectUsersError {
    Forbidden,
    NotFound,
    BadRequest,
    DatabaseError,
}

impl std::fmt::Display for GetProjectUsersError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetProjectUsersError::Forbidden => write!(f, "Forbidden."),
            GetProjectUsersError::NotFound => write!(f, "Not found."),
            GetProjectUsersError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            GetProjectUsersError::BadRequest => {
                write!(f, "Bad request.")
            }
        }
    }
}

impl ResponseError for GetProjectUsersError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetProjectUsersError::Forbidden => StatusCode::FORBIDDEN,
            GetProjectUsersError::NotFound => StatusCode::NOT_FOUND,
            GetProjectUsersError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            GetProjectUsersError::BadRequest => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_get_project_users(
    state: web::Data<AppState>,
    project_id: u64,
    page: Page,
) -> Result<GetProjectUsersResultView, GetProjectUsersError> {
    let view = GetProjectUsersQueryView::new(project_id, page.limit, page.offset);
    let result: PagedRows<ProjectMemberRow> =
        state.get_smart_db().fetch_one(&view).await.map_err(|e| {
            log_db_error("projects/project_id/users/get", &e);
            GetProjectUsersError::DatabaseError
        })?;

    Ok(GetProjectUsersResultView {
        users: result.items.into_iter().map(User::from).collect(),
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
    summary = "List the members of a project",
    description = "Returns one page of the users attached to the project, sorted by name, with \
                   their Core API id and full name. Being able to see the project is enough.\n\n\
                   `limit` (default 100, at most 500) and `offset` (default 0) select the page; \
                   `total` is the number of members, whatever the page.\n\n\
                   `name` may be `null` when the name could not be resolved; the id is always \
                   present and gives the profile through Core API `GET /api/v1/user/{id}/`.",
    responses(
        (
            status = 200,
            description = "One page of the members. Empty if the project has none, or if `offset` is past the end.",
            body = GetProjectUsersResultView,
            example = json!({
                "users": [
                    { "id": 42, "name": "Jean Dupont" },
                    { "id": 51, "name": "Amina Bensaïd" }
                ],
                "total": 2
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
            description = "En-tête `Authorization` absent, JWT invalide ou expiré, ou session révoquée.",
            body = String,
            content_type = "text/plain",
            example = json!("Jeton expiré")
        ),
        (
            status = 404,
            description = "Projet inexistant, ou invisible pour l'appelant — les deux cas sont volontairement indiscernables.",
            body = String,
            content_type = "text/plain",
            example = json!("Not found.")
        ),
        (
            status = 500,
            description = "Erreur de base de données.",
            body = String,
            content_type = "text/plain",
            example = json!("An error occurred while accessing the database.")
        ),
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Users",
)]
#[get("/")]
pub async fn get_project_users(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<ProjectPathParams>,
    page: web::Query<PageParams>,
) -> Result<impl Responder, GetProjectUsersError> {
    require_access(
        &state,
        auth_user.id,
        params.project_id(),
        None,
        Requirement::ViewProject,
    )
    .await?;
    let result = trigger_get_project_users(state, params.project_id, page.page()).await?;
    Ok(HttpResponse::Ok().json(result))
}

impl From<AccessDenied> for GetProjectUsersError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => GetProjectUsersError::NotFound,
            AccessDenied::Forbidden => GetProjectUsersError::Forbidden,
            AccessDenied::DatabaseError => GetProjectUsersError::DatabaseError,
        }
    }
}
