use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

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
) -> Result<GetProjectUsersResultView, GetProjectUsersError> {
    let view = GetProjectUsersQueryView::new(project_id);
    let result: Vec<ProjectMemberRow> = state
        .get_smart_db()
        .fetch_all(&view)
        .await
        .map_err(|_| GetProjectUsersError::DatabaseError)?;

    Ok(GetProjectUsersResultView {
        users: result.into_iter().map(User::from).collect(),
    })
}

#[utoipa::path(
    get,
    params(
        ProjectPathParams,
    ),
    path = "",
    summary = "Lister les membres d'un projet",
    description = "Renvoie les utilisateurs rattachés au projet, avec leur identifiant Core API et \
                   leur nom complet. Il suffit d'être membre du projet.\n\n\
                   `name` peut être `null` si le nom n'a pas pu être résolu côté Core API ; \
                   l'identifiant, lui, est toujours présent et permet d'aller chercher la fiche \
                   via `GET /api/v1/user/{id}/` de Core API.",
    responses(
        (
            status = 200,
            description = "Membres du projet.",
            body = GetProjectUsersResultView,
            example = json!({
                "users": [
                    { "id": 42, "name": "Jean Dupont" },
                    { "id": 51, "name": "Amina Bensaïd" }
                ]
            })
        ),
        (
            status = 400,
            description = "Un segment de l'URL n'est pas un entier, ou le corps JSON est malformé.",
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
) -> Result<impl Responder, GetProjectUsersError> {
    require_access(
        &state,
        auth_user.id,
        params.project_id(),
        None,
        Requirement::ViewProject,
    )
    .await?;
    let result = trigger_get_project_users(state, params.project_id).await?;
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
