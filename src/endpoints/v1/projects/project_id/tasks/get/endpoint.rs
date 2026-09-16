use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::tasks::get_project_tasks::view::{GetProjectTasksQueryView, Task};
use crate::endpoints::v1::projects::access::{require_access, AccessDenied, Requirement};
use crate::endpoints::v1::projects::project_id::tasks::get::view::GetTasksResultView;
use crate::endpoints::v1::projects::project_id::ProjectPathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum GetTasksError {
    Forbidden,
    NotFound,
    BadRequest,
    DatabaseError,
}

impl std::fmt::Display for GetTasksError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetTasksError::Forbidden => write!(f, "Forbidden."),
            GetTasksError::NotFound => write!(f, "Not found."),
            GetTasksError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            GetTasksError::BadRequest => {
                write!(f, "Bad request.")
            }
        }
    }
}

impl ResponseError for GetTasksError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetTasksError::Forbidden => StatusCode::FORBIDDEN,
            GetTasksError::NotFound => StatusCode::NOT_FOUND,
            GetTasksError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            GetTasksError::BadRequest => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_get_project_tasks(
    state: web::Data<AppState>,
    project_id: u64,
) -> Result<GetTasksResultView, GetTasksError> {
    let view = GetProjectTasksQueryView::new(project_id);
    let result: Vec<Task> = state
        .get_smart_db()
        .fetch_all(&view)
        .await
        .map_err(|_| GetTasksError::DatabaseError)?;

    Ok(GetTasksResultView {
        tasks: result.into_iter().map(|t| t.into()).collect(),
    })
}

#[utoipa::path(
    get,
    params(
        ProjectPathParams,
    ),
    path = "",
    summary = "Lister les tâches d'un projet",
    description = "Renvoie toutes les tâches du projet, avec leur statut, leur priorité, leur \
                   échéance, leur agent assigné et leurs champs personnalisés. Il suffit d'être \
                   membre du projet.\n\n\
                   `GET /api/v1/projects/{project_id}/` renvoie déjà ces mêmes tâches avec le \
                   projet et ses membres : cet endpoint sert à rafraîchir la seule liste des \
                   tâches.\n\n\
                   Le champ `description` est toujours une chaîne vide : la table des tâches n'en \
                   stocke pas.",
    responses(
        (
            status = 200,
            description = "Tâches du projet. Vide si le projet n'en a aucune.",
            body = GetTasksResultView,
            example = json!({
                "tasks": [
                    {
                        "id": 77,
                        "title": "Consulter les riverains",
                        "description": "",
                        "status": "InProgress",
                        "priority": "High",
                        "due_date": "2026-10-15T00:00:00Z",
                        "assigned_to": 42,
                        "fields": [
                            { "label": "Budget engagé", "task_type": "Number", "fields_options": [] }
                        ]
                    }
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
    tag = "Tasks",
)]
#[get("/")]
pub async fn get_project_tasks(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<ProjectPathParams>,
) -> Result<impl Responder, GetTasksError> {
    require_access(
        &state,
        auth_user.id,
        params.project_id(),
        None,
        Requirement::ViewProject,
    )
    .await?;
    let result = trigger_get_project_tasks(state, params.project_id).await?;
    Ok(HttpResponse::Ok().json(result))
}

impl From<AccessDenied> for GetTasksError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => GetTasksError::NotFound,
            AccessDenied::Forbidden => GetTasksError::Forbidden,
            AccessDenied::DatabaseError => GetTasksError::DatabaseError,
        }
    }
}
