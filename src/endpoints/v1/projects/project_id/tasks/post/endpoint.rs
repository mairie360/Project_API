use actix_web::http::StatusCode;
use actix_web::{post, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::tasks::create_task::view::{
    CreateTaskQueryView, TaskPriority as DbTaskPriority, TaskStatus,
};
use crate::endpoints::v1::projects::access::{require_access, AccessDenied, Requirement};
use crate::endpoints::v1::projects::project_id::get::view::TaskPriority as ApiTaskPriority;
use crate::endpoints::v1::projects::project_id::tasks::post::view::{
    CreateTaskResultView, CreateTaskView,
};
use crate::endpoints::v1::projects::project_id::ProjectPathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum CreateTaskError {
    Forbidden,
    NotFound,
    DatabaseError,
    BadRequest,
}

impl std::fmt::Display for CreateTaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CreateTaskError::Forbidden => write!(f, "Forbidden."),
            CreateTaskError::NotFound => write!(f, "Not found."),
            CreateTaskError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            CreateTaskError::BadRequest => {
                write!(f, "Bad request.")
            }
        }
    }
}

impl ResponseError for CreateTaskError {
    fn status_code(&self) -> StatusCode {
        match self {
            CreateTaskError::Forbidden => StatusCode::FORBIDDEN,
            CreateTaskError::NotFound => StatusCode::NOT_FOUND,
            CreateTaskError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            CreateTaskError::BadRequest => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_create_task(
    state: web::Data<AppState>,
    _user_id: u64,
    project_id: u64,
    view: CreateTaskView,
) -> Result<CreateTaskResultView, CreateTaskError> {
    let name = view.name().to_string();

    // Statut et priorité facultatifs : valeurs par défaut de la table (todo, medium).
    let status = view
        .status()
        .map(|status| status.to_string().into())
        .unwrap_or(TaskStatus::Todo);
    let priority = match view.priority() {
        Some(ApiTaskPriority::Urgent) => DbTaskPriority::High,
        Some(priority) => priority.to_string().into(),
        None => DbTaskPriority::Medium,
    };
    if status == TaskStatus::Error || priority == DbTaskPriority::Error {
        return Err(CreateTaskError::BadRequest);
    }
    let query_view = CreateTaskQueryView::new(
        project_id,
        &name,
        status,
        priority,
        *view.due_date(),
        *view.assigned_to(),
        view.fields(),
    );
    let result: i32 = state
        .get_smart_db()
        .fetch_scalar::<i32, _>(&query_view)
        .await
        .map_err(|_| CreateTaskError::DatabaseError)?;

    Ok(CreateTaskResultView {
        task_id: result as u64,
        name: name.to_string(),
        description: None,
    })
}

#[utoipa::path(
    post,
    params(
        ProjectPathParams,
    ),
    path = "",
    summary = "Créer une tâche",
    description = "Ajoute une tâche au projet. Réservé aux responsables du projet : un agent \
                   simplement assigné à d'autres tâches ne peut pas en créer.\n\n\
                   `status` et `priority` sont facultatifs et prennent leur valeur par défaut si \
                   absents. `fields` porte les champs personnalisés du projet et doit être présent, \
                   quitte à être un tableau vide.\n\n\
                   La description passée ici n'est pas persistée : elle est renvoyée dans la \
                   réponse, mais les lectures ultérieures la donneront vide.",
    responses(
        (
            status = 200,
            description = "Tâche créée.",
            body = CreateTaskResultView,
            example = json!({
                "task_id": 77,
                "name": "Consulter les riverains",
                "description": "Réunion publique à organiser avant le 15 octobre"
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
            status = 403,
            description = "Projet visible par l'appelant, mais droits insuffisants pour cette opération.",
            body = String,
            content_type = "text/plain",
            example = json!("Forbidden.")
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
    request_body(
        content = CreateTaskView,
        description = "Définition de la tâche. `fields` est obligatoire, même vide.",
        example = json!({
            "name": "Consulter les riverains",
            "description": "Réunion publique à organiser avant le 15 octobre",
            "due_date": "2026-10-15T00:00:00Z",
            "status": "Todo",
            "priority": "High",
            "assigned_to": 42,
            "fields": []
        })
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Tasks",
)]
#[post("/")]
pub async fn create_task(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    view: web::Json<CreateTaskView>,
    params: web::Path<ProjectPathParams>,
) -> Result<impl Responder, CreateTaskError> {
    require_access(
        &state,
        auth_user.id,
        params.project_id(),
        None,
        Requirement::ManageProject,
    )
    .await?;
    let view = view.try_into().map_err(|_| CreateTaskError::BadRequest)?;
    let result = trigger_create_task(state, auth_user.id, params.project_id, view).await?;
    Ok(HttpResponse::Ok().json(result))
}

impl From<AccessDenied> for CreateTaskError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => CreateTaskError::NotFound,
            AccessDenied::Forbidden => CreateTaskError::Forbidden,
            AccessDenied::DatabaseError => CreateTaskError::DatabaseError,
        }
    }
}
