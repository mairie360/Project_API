use actix_web::http::StatusCode;
use actix_web::{post, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::database::error::DbError;
use mairie360_api_lib::error::ApiLibError;
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::smart_db::SmartTransaction;
use mairie360_api_lib::state::AppState;

use crate::endpoints::db_error::log_db_error;

use crate::database::tasks::create_task::view::{
    CreateTaskQueryView, TaskPriority as DbTaskPriority, TaskStatus, ASSIGNEE_NOT_IN_PROJECT,
};
use crate::endpoints::v1::projects::access::{begin_write, commit, AccessDenied, Requirement};
use crate::endpoints::v1::projects::project_id::get::view::TaskPriority as ApiTaskPriority;
use crate::endpoints::v1::projects::project_id::tasks::post::view::{
    CreateTaskResultView, CreateTaskView,
};
use crate::endpoints::v1::projects::project_id::ProjectPathParams;
use crate::endpoints::validation::ValidatedJson;

#[derive(Debug, Clone, PartialEq)]
pub enum CreateTaskError {
    Forbidden,
    NotFound,
    DatabaseError,
    BadRequest,
    UnknownAssignee,
    AssigneeNotInProject,
}

impl std::fmt::Display for CreateTaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CreateTaskError::Forbidden => write!(f, "Forbidden."),
            CreateTaskError::NotFound => write!(f, "Not found."),
            CreateTaskError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            CreateTaskError::UnknownAssignee => write!(f, "`assigned_to` does not match any user."),
            CreateTaskError::AssigneeNotInProject => {
                write!(f, "`assigned_to` is not a member of the project.")
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
            CreateTaskError::UnknownAssignee => StatusCode::BAD_REQUEST,
            CreateTaskError::AssigneeNotInProject => StatusCode::BAD_REQUEST,
            CreateTaskError::BadRequest => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_create_task(
    tx: &mut SmartTransaction,
    user_id: u64,
    project_id: u64,
    view: CreateTaskView,
) -> Result<CreateTaskResultView, CreateTaskError> {
    let name = view.name().to_string();

    // Optional status and priority: defaults of the table (todo, medium).
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
    let description = view
        .description()
        .as_deref()
        .unwrap_or_default()
        .to_string();
    let query_view = CreateTaskQueryView::new(
        project_id,
        user_id,
        &name,
        &description,
        status,
        priority,
        *view.due_date(),
        *view.assigned_to(),
        view.fields(),
    );
    let result: i32 = tx
        .fetch_scalar::<i32, _>(&query_view)
        .await
        .map_err(|e| match e {
            ApiLibError::Database(DbError::ForeignKeyViolation(_)) => {
                CreateTaskError::UnknownAssignee
            }
            e => {
                log_db_error("projects/project_id/tasks/post", &e);
                CreateTaskError::DatabaseError
            }
        })?;

    if result == ASSIGNEE_NOT_IN_PROJECT {
        return Err(CreateTaskError::AssigneeNotInProject);
    }

    Ok(CreateTaskResultView {
        task_id: result as u64,
        name,
        description,
    })
}

#[utoipa::path(
    post,
    params(
        ProjectPathParams,
    ),
    path = "",
    summary = "Create a task",
    description = "Adds a task to the project. Reserved to the project managers: an agent merely \
                   assigned to other tasks cannot create one.\n\n\
                   `status` and `priority` are optional and take their default value when absent. \
                   `description` is optional (empty string when absent). `fields` carries the \
                   custom fields of the project and must be present, even as an empty array.\n\n\
                   `assigned_to` must be the owner or a member of the project (add them with \
                   `POST …/users/` first), otherwise the task would be invisible to its \
                   assignee.\n\n\
                   The creation is logged as a `task_created` history entry signed by the caller \
                   (see `GET …/tasks/{task_id}/collaboration`).",
    responses(
        (
            status = 200,
            description = "Task created.",
            body = CreateTaskResultView,
            example = json!({
                "task_id": 77,
                "name": "Consulter les riverains",
                "description": "Réunion publique à organiser avant le 15 octobre"
            })
        ),
        (
            status = 400,
            description = "A URL segment is not an integer, malformed JSON body, `assigned_to` that does not match any user or is neither the owner nor a member of the project, or a field breaking its rules: `name` 1 to 255 characters, not blank, no control character, no `<` or `>`; `description` at most 5000 characters, no `<` or `>`, no control character other than line breaks and tabs; each `fields[].label` 1 to 255 characters (same rules as `name`) and each `fields_options[].option` string free of `<`, `>` and control characters.",
            body = String,
            content_type = "text/plain",
            example = json!("`assigned_to` is not a member of the project.")
        ),
        (
            status = 401,
            description = "Missing `Authorization` header, invalid or expired JWT, or revoked session.",
            body = String,
            content_type = "text/plain",
            example = json!("Jeton expiré")
        ),
        (
            status = 403,
            description = "Project visible to the caller, but insufficient rights for this operation.",
            body = String,
            content_type = "text/plain",
            example = json!("Forbidden.")
        ),
        (
            status = 404,
            description = "Unknown project, or project not visible to the caller — both cases are deliberately indistinguishable.",
            body = String,
            content_type = "text/plain",
            example = json!("Not found.")
        ),
        (
            status = 500,
            description = "Database error.",
            body = String,
            content_type = "text/plain",
            example = json!("An error occurred while accessing the database.")
        ),
    ),
    request_body(
        content = CreateTaskView,
        description = "Definition of the task. `fields` is required, even empty.",
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
    view: ValidatedJson<CreateTaskView>,
    params: web::Path<ProjectPathParams>,
) -> Result<impl Responder, CreateTaskError> {
    let (mut tx, _) = begin_write(
        &state,
        auth_user.id,
        params.project_id(),
        None,
        Requirement::ManageProject,
    )
    .await?;
    let view = view.into_inner();
    let result = trigger_create_task(&mut tx, auth_user.id, params.project_id, view).await?;
    commit(tx, "projects/project_id/tasks/post").await?;
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
