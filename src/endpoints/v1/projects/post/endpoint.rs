use actix_web::http::StatusCode;
use actix_web::{post, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::project::create::view::CreateProjectQueryView;
use crate::endpoints::v1::projects::access::{require_manager_role, AccessDenied};
use crate::endpoints::v1::projects::post::view::{CreateProjectResultView, CreateProjectView};
use crate::endpoints::validation::ValidatedJson;

#[derive(Debug, Clone, PartialEq)]
pub enum CreateProjectError {
    Forbidden,
    NotFound,
    DatabaseError,
}

impl std::fmt::Display for CreateProjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CreateProjectError::Forbidden => write!(f, "Forbidden."),
            CreateProjectError::NotFound => write!(f, "Not found."),
            CreateProjectError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for CreateProjectError {
    fn status_code(&self) -> StatusCode {
        match self {
            CreateProjectError::Forbidden => StatusCode::FORBIDDEN,
            CreateProjectError::NotFound => StatusCode::NOT_FOUND,
            CreateProjectError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_create_project(
    state: web::Data<AppState>,
    user_id: u64,
    view: CreateProjectView,
) -> Result<i32, CreateProjectError> {
    let view = CreateProjectQueryView::new(view.name(), view.description(), user_id);
    let result: i32 = state
        .get_smart_db()
        .fetch_scalar::<i32, _>(&view)
        .await
        .map_err(|_| CreateProjectError::DatabaseError)?;

    Ok(result)
}

#[utoipa::path(
    post,
    path = "",
    summary = "Créer un projet",
    description = "Crée un projet dont l'appelant devient responsable. Réservé aux rôles Admin, \
                   Maire et Responsable : un agent sans l'un de ces rôles reçoit `403`.\n\n\
                   Rattacher un `group_id` donne d'emblée accès au projet à tous les membres du \
                   groupe, tel que Core API le définit.\n\n\
                   La réponse ne contient que l'identifiant attribué ; relire le projet complet \
                   avec `GET /api/v1/projects/{project_id}/`.",
    responses(
        (
            status = 200,
            description = "Projet créé. Le corps contient l'identifiant attribué.",
            body = CreateProjectResultView,
            example = json!({ "project_id": 12 })
        ),
        (
            status = 400,
            description = "Malformed JSON body, missing `name`, or a field breaking its rules: `name` 1 to 255 characters, not blank, no control character, no `<` or `>`; `description` at most 5000 characters, no `<` or `>`, no control character other than line breaks and tabs. The body names the first invalid field.",
            body = String,
            content_type = "text/plain",
            example = json!("Invalid `name`: must not contain `<` or `>`")
        ),
        (
            status = 401,
            description = "En-tête `Authorization` absent, JWT invalide ou expiré, ou session révoquée.",
            body = String,
            content_type = "text/plain",
            example = json!("Jeton expiré")
        ),
        (
            status = 403,
            description = "L'appelant n'a ni le rôle Admin, ni Maire, ni Responsable.",
            body = String,
            content_type = "text/plain",
            example = json!("Forbidden.")
        ),
        (
            status = 500,
            description = "Erreur de base de données.",
            body = String,
            content_type = "text/plain",
            example = json!("An error occurred while accessing the database.")
        ),
    ),
    request_body(
        content = CreateProjectView,
        description = "Nom du projet, description et groupe facultatifs.",
        example = json!({
            "name": "Réfection de la place du marché",
            "description": "Travaux de voirie 2026",
            "group_id": 3
        })
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Projects",
)]
#[post("/")]
pub async fn create_project(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    view: ValidatedJson<CreateProjectView>,
) -> Result<impl Responder, CreateProjectError> {
    require_manager_role(&state, auth_user.id).await?;
    let view = view.into_inner();
    let project_id = trigger_create_project(state, auth_user.id, view).await?;
    Ok(HttpResponse::Ok().json(CreateProjectResultView {
        project_id: project_id as u64,
    }))
}

impl From<AccessDenied> for CreateProjectError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => CreateProjectError::NotFound,
            AccessDenied::Forbidden => CreateProjectError::Forbidden,
            AccessDenied::DatabaseError => CreateProjectError::DatabaseError,
        }
    }
}
