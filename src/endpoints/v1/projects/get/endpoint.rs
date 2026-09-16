use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

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
) -> Result<GetProjectsResultView, GetProjectsError> {
    let view = GetProjectsQueryView::new(user_id);
    let result: Vec<ProjectView> = state
        .get_smart_db()
        .fetch_all(&view)
        .await
        .map_err(|_| GetProjectsError::DatabaseError)?;

    Ok(GetProjectsResultView {
        projects: result.into_iter().map(|p| p.into()).collect(),
    })
}

#[utoipa::path(
    get,
    path = "",
    summary = "Lister ses projets",
    description = "Renvoie les projets visibles par l'utilisateur porté par le JWT. Vue de \
                   liste : ni les tâches ni les membres ne sont inclus, il faut passer par \
                   `GET /api/v1/projects/{project_id}/` pour le détail d'un projet.\n\n\
                   La liste est vide si l'utilisateur n'a accès à aucun projet.",
    responses(
        (
            status = 200,
            description = "Projets visibles par l'utilisateur connecté.",
            body = GetProjectsResultView,
            example = json!({
                "projects": [
                    { "id": 12, "name": "Réfection de la place du marché", "description": "Travaux de voirie 2026", "status": "Active" },
                    { "id": 18, "name": "Numérisation de l'état civil", "description": "", "status": "Completed" }
                ]
            })
        ),
        (
            status = 401,
            description = "En-tête `Authorization` absent, JWT invalide ou expiré, ou session révoquée.",
            body = String,
            content_type = "text/plain",
            example = json!("Jeton expiré")
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
    tag = "Projects",
)]
#[get("/")]
pub async fn get_projects(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl Responder, GetProjectsError> {
    let result = trigger_get_projects(state, auth_user.id).await?;
    Ok(HttpResponse::Ok().json(result))
}
