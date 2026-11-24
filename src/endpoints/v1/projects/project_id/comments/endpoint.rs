use actix_web::http::StatusCode;
use actix_web::{post, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::smart_db::SmartTransaction;
use mairie360_api_lib::state::AppState;

use crate::database::activity::view::{ActivityRow, AddProjectCommentQueryView};
use crate::endpoints::db_error::{classify_db_error, DbFailure};
use crate::endpoints::v1::projects::access::{
    begin_write, commit, AccessDenied, Requirement, PROJECT_CLOSED_MESSAGE,
};
use crate::endpoints::v1::projects::project_id::activity::view::ActivityItemView;
use crate::endpoints::v1::projects::project_id::comments::view::AddProjectCommentView;
use crate::endpoints::v1::projects::project_id::ProjectPathParams;
use crate::endpoints::validation::ValidatedJson;

#[derive(Debug, Clone, PartialEq)]
pub enum AddProjectCommentError {
    NotFound,
    /// The project is closed: read-only until a manager reopens it.
    ProjectClosed,
    /// The database refused the message (its `CHECK`).
    BadRequest,
    DatabaseError,
}

impl std::fmt::Display for AddProjectCommentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddProjectCommentError::NotFound => write!(f, "Not found."),
            AddProjectCommentError::ProjectClosed => f.write_str(PROJECT_CLOSED_MESSAGE),
            AddProjectCommentError::BadRequest => {
                write!(f, "Invalid `message`: must hold 1 to 2000 characters")
            }
            AddProjectCommentError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for AddProjectCommentError {
    fn status_code(&self) -> StatusCode {
        match self {
            AddProjectCommentError::NotFound => StatusCode::NOT_FOUND,
            AddProjectCommentError::ProjectClosed => StatusCode::CONFLICT,
            AddProjectCommentError::BadRequest => StatusCode::BAD_REQUEST,
            AddProjectCommentError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

impl From<AccessDenied> for AddProjectCommentError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            // Seeing the project is enough to comment it: no `Forbidden` is possible.
            AccessDenied::NotFound | AccessDenied::Forbidden => AddProjectCommentError::NotFound,
            AccessDenied::Closed => AddProjectCommentError::ProjectClosed,
            AccessDenied::DatabaseError => AddProjectCommentError::DatabaseError,
        }
    }
}

async fn trigger_add_project_comment(
    tx: &mut SmartTransaction,
    project_id: u64,
    user_id: u64,
    view: &AddProjectCommentView,
) -> Result<ActivityItemView, AddProjectCommentError> {
    let row: ActivityRow = tx
        .fetch_one(&AddProjectCommentQueryView::new(
            project_id,
            user_id,
            &view.message,
        ))
        .await
        .map_err(
            |e| match classify_db_error("projects/project_id/comments/post", &e) {
                DbFailure::CheckViolation => AddProjectCommentError::BadRequest,
                _ => AddProjectCommentError::DatabaseError,
            },
        )?;
    Ok(row.into())
}

#[utoipa::path(
    post,
    params(ProjectPathParams),
    path = "",
    summary = "Comment on a project",
    description = "Adds a comment to the project (MAIR-1089), signed by the caller (from the JWT, \
                   never from the body). Anyone who sees the project may comment it. The message \
                   is trimmed, then must hold 1 to 2000 characters. The comment is answered as an \
                   entry of the activity (`kind` `comment`) and then appears in \
                   `GET …/activity/`.",
    request_body(
        content = AddProjectCommentView,
        description = "Text of the comment.",
        example = json!({ "message": "The tender closes on Friday." })
    ),
    responses(
        (status = 201, description = "Comment added.", body = ActivityItemView,
         example = json!({ "kind": "comment", "id": 31, "task_id": null, "task_title": null, "author_id": 42,
                           "author_name": "Jean Dupont", "message": "The tender closes on Friday.", "changes": null,
                           "label": "Comment", "created_at": "2026-11-21T09:30:00.000Z" })),
        (status = 400, description = "A URL segment is not an integer, malformed JSON body, or `message` empty or blank, longer than 2000 characters once trimmed, or holding a control character other than line breaks and tabs.",
         body = String, content_type = "text/plain", example = json!("Invalid `message`: must not be empty")),
        (status = 401, description = "Missing `Authorization` header, invalid or expired JWT, or revoked session.",
         body = String, content_type = "text/plain", example = json!("Jeton expiré")),
        (status = 404, description = "Unknown project, or project not visible to the caller — both cases are deliberately indistinguishable.",
         body = String, content_type = "text/plain", example = json!("Not found.")),
        (status = 409, description = "The caller may write, but the project is closed (`Completed`): it is read-only until a manager reopens it.",
         body = String, content_type = "text/plain", example = json!("Project is closed.")),
        (status = 500, description = "Database error.", body = String, content_type = "text/plain",
         example = json!("An error occurred while accessing the database.")),
    ),
    security(("jwt" = [])),
    tag = "Activity",
)]
#[post("/")]
pub async fn add_project_comment(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<ProjectPathParams>,
    view: ValidatedJson<AddProjectCommentView>,
) -> Result<impl Responder, AddProjectCommentError> {
    const OPERATION: &str = "projects/project_id/comments/post";
    let view = view.into_inner();
    let (mut tx, _) = begin_write(
        &state,
        auth_user.id,
        params.project_id(),
        None,
        Requirement::ViewProject,
    )
    .await?;
    let comment =
        trigger_add_project_comment(&mut tx, params.project_id(), auth_user.id, &view).await?;
    commit(tx, OPERATION).await?;
    Ok(HttpResponse::Created().json(comment))
}
