use actix_web::http::StatusCode;
use actix_web::{patch, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::tasks::create_task::view::{
    TaskPriority as DbTaskPriority, TaskStatus as DbTaskStatus,
};
use crate::database::tasks::patch_task::view::PatchTaskQueryView;
use crate::endpoints::v1::projects::access::{require_access, AccessDenied, Requirement};
use crate::endpoints::v1::projects::project_id::get::view::TaskPriority;
use crate::endpoints::v1::projects::project_id::tasks::task_id::patch::view::PatchTaskView;
use crate::endpoints::v1::projects::project_id::tasks::task_id::TaskPathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum PatchTaskError {
    Forbidden,
    BadRequest,
    NotFound,
    DatabaseError,
}

impl std::fmt::Display for PatchTaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PatchTaskError::Forbidden => write!(f, "Forbidden."),
            PatchTaskError::BadRequest => write!(f, "Bad request."),
            PatchTaskError::NotFound => write!(f, "Unknown task."),
            PatchTaskError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for PatchTaskError {
    fn status_code(&self) -> StatusCode {
        match self {
            PatchTaskError::Forbidden => StatusCode::FORBIDDEN,
            PatchTaskError::BadRequest => StatusCode::BAD_REQUEST,
            PatchTaskError::NotFound => StatusCode::NOT_FOUND,
            PatchTaskError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_patch_task(
    state: web::Data<AppState>,
    project_id: u64,
    task_id: u64,
    view: PatchTaskView,
) -> Result<(), PatchTaskError> {
    if view
        .name
        .as_deref()
        .is_some_and(|name| name.trim().is_empty())
    {
        return Err(PatchTaskError::BadRequest);
    }
    let status = view
        .status
        .map(|status| DbTaskStatus::from(status.to_string()));
    // La base ne connaît pas « urgent » : la priorité la plus haute est retenue.
    let priority = view.priority.map(|priority| match priority {
        TaskPriority::Urgent => DbTaskPriority::High,
        priority => DbTaskPriority::from(priority.to_string()),
    });
    if status == Some(DbTaskStatus::Error) || priority == Some(DbTaskPriority::Error) {
        return Err(PatchTaskError::BadRequest);
    }

    let updated: bool = state
        .get_smart_db()
        .fetch_scalar(&PatchTaskQueryView::new(
            project_id,
            task_id,
            view.name.as_deref().map(str::trim),
            status,
            priority,
            view.due_date,
            view.assigned_to,
        ))
        .await
        .map_err(|_| PatchTaskError::DatabaseError)?;

    if updated {
        Ok(())
    } else {
        Err(PatchTaskError::NotFound)
    }
}

#[utoipa::path(
    patch,
    path = "",
    summary = "Modifier une tâche",
    description = "Met à jour partiellement une tâche : un champ absent reste inchangé.\n\n\
                   Deux niveaux de droits se superposent ici. Un **responsable du projet** peut \
                   tout modifier. Un **agent assigné à la tâche** ne peut changer que son statut : \
                   si son corps de requête touche à autre chose, la réponse est `403`.\n\n\
                   `assigned_to` distingue l'absence du `null` : omettre le champ conserve \
                   l'assignation, l'envoyer à `null` la retire.\n\n\
                   Deux champs sont acceptés mais **non persistés** par cette opération : \
                   `description`, que la table des tâches ne stocke pas, et `fields`. La réponse a \
                   un corps vide.",
    responses(
        (
            status = 204,
            description = "Tâche mise à jour. Corps vide.",
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
            description = "Droits insuffisants sur la tâche, ou agent assigné tentant de modifier autre chose que le statut.",
            body = String,
            content_type = "text/plain",
            example = json!("Forbidden.")
        ),
        (
            status = 404,
            description = "Projet ou tâche inexistant, ou projet invisible pour l'appelant.",
            body = String,
            content_type = "text/plain",
            example = json!("Unknown task.")
        ),
        (
            status = 500,
            description = "Erreur de base de données.",
            body = String,
            content_type = "text/plain",
            example = json!("An error occurred while accessing the database.")
        ),
    ),
    params(
        TaskPathParams
    ),
    request_body(
        content = PatchTaskView,
        description = "Champs à modifier. Tous facultatifs.",
        example = json!({ "status": "Completed" })
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Tasks",
)]
#[patch("/")]
pub async fn patch_task(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<TaskPathParams>,
    view: web::Json<PatchTaskView>,
) -> Result<impl Responder, PatchTaskError> {
    let view = view.into_inner();
    let access = require_access(
        &state,
        auth_user.id,
        params.project_id(),
        Some(params.task_id()),
        Requirement::ActOnTask,
    )
    .await?;
    // L'agent assigné qui ne gère pas le projet ne peut modifier que le statut de sa tâche.
    if !access.can_manage() && !view.only_changes_status() {
        return Err(PatchTaskError::Forbidden);
    }
    trigger_patch_task(state, params.project_id(), params.task_id(), view).await?;
    Ok(HttpResponse::NoContent().finish())
}

impl From<AccessDenied> for PatchTaskError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => PatchTaskError::NotFound,
            AccessDenied::Forbidden => PatchTaskError::Forbidden,
            AccessDenied::DatabaseError => PatchTaskError::DatabaseError,
        }
    }
}
