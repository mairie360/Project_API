use actix_web::http::StatusCode;
use actix_web::{post, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::tasks::collaboration::view::{AddTaskCommentQueryView, TaskComment};
use crate::endpoints::v1::projects::access::{require_access, AccessDenied, Requirement};
use crate::endpoints::v1::projects::project_id::tasks::task_id::comments::view::{
    AddTaskCommentView, MAX_COMMENT_LENGTH,
};
use crate::endpoints::v1::projects::project_id::tasks::task_id::TaskPathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum AddTaskCommentError {
    Forbidden,
    BadRequest,
    NotFound,
    DatabaseError,
}

impl std::fmt::Display for AddTaskCommentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddTaskCommentError::Forbidden => write!(f, "Forbidden."),
            AddTaskCommentError::BadRequest => write!(f, "Bad request."),
            AddTaskCommentError::NotFound => write!(f, "Unknown task."),
            AddTaskCommentError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for AddTaskCommentError {
    fn status_code(&self) -> StatusCode {
        match self {
            AddTaskCommentError::Forbidden => StatusCode::FORBIDDEN,
            AddTaskCommentError::BadRequest => StatusCode::BAD_REQUEST,
            AddTaskCommentError::NotFound => StatusCode::NOT_FOUND,
            AddTaskCommentError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_add_task_comment(
    state: web::Data<AppState>,
    params: &TaskPathParams,
    user_id: u64,
    view: AddTaskCommentView,
) -> Result<TaskComment, AddTaskCommentError> {
    let message = view.message.trim();
    if message.is_empty() || message.chars().count() > MAX_COMMENT_LENGTH {
        return Err(AddTaskCommentError::BadRequest);
    }

    let comments: Vec<TaskComment> = state
        .get_smart_db()
        .fetch_all(&AddTaskCommentQueryView::new(
            params.project_id(),
            params.task_id(),
            user_id,
            message,
        ))
        .await
        .map_err(|_| AddTaskCommentError::DatabaseError)?;

    comments
        .into_iter()
        .next()
        .ok_or(AddTaskCommentError::NotFound)
}

#[utoipa::path(
    post,
    path = "comments",
    summary = "Commenter une tâche",
    description = "Ajoute un commentaire au fil de discussion d'une tâche. Ouvert aux responsables \
                   du projet et à l'agent assigné à la tâche.\n\n\
                   Le message est nettoyé de ses espaces de bord, puis doit contenir entre 1 et \
                   2000 caractères : un message vide ou trop long est refusé en `400`.\n\n\
                   L'auteur est déduit du JWT, jamais du corps. Le commentaire créé est renvoyé \
                   avec son identifiant et sa date, et apparaît ensuite dans \
                   `GET …/tasks/{task_id}/collaboration`.",
    params(
        TaskPathParams
    ),
    request_body(
        content = AddTaskCommentView,
        description = "Texte du commentaire.",
        example = json!({ "message": "La réunion publique est calée au 3 octobre." })
    ),
    responses(
        (
            status = 201,
            description = "Commentaire ajouté.",
            body = TaskComment,
            example = json!({
                "id": "c-1",
                "message": "La réunion publique est calée au 3 octobre.",
                "author": { "id": "user-42", "name": "Jean Dupont" },
                "createdAt": "2026-09-14T09:12:00Z"
            })
        ),
        (
            status = 400,
            description = "Corps JSON malformé, segment d'URL non entier, ou message vide ou de plus de 2000 caractères.",
            body = String,
            content_type = "text/plain",
            example = json!("Bad request.")
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
            description = "Ni responsable du projet, ni agent assigné à la tâche.",
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
    security(
        ("jwt" = [])
    ),
    tag = "Tasks",
)]
#[post("/comments")]
pub async fn add_task_comment(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<TaskPathParams>,
    view: web::Json<AddTaskCommentView>,
) -> Result<impl Responder, AddTaskCommentError> {
    require_access(
        &state,
        auth_user.id,
        params.project_id(),
        Some(params.task_id()),
        Requirement::ActOnTask,
    )
    .await?;
    let comment = trigger_add_task_comment(state, &params, auth_user.id, view.into_inner()).await?;
    Ok(HttpResponse::Created().json(comment))
}

impl From<AccessDenied> for AddTaskCommentError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => AddTaskCommentError::NotFound,
            AccessDenied::Forbidden => AddTaskCommentError::Forbidden,
            AccessDenied::DatabaseError => AddTaskCommentError::DatabaseError,
        }
    }
}
