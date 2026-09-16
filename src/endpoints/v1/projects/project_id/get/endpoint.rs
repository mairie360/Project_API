use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::project::get_project::view::GetVisibleProjectQueryView;
use crate::database::project::get_projects::view::ProjectView;
use crate::database::tasks::get_project_tasks::view::{GetProjectTasksQueryView, Task};
use crate::database::users::get_project_users::view::{GetProjectUsersQueryView, ProjectMemberRow};
use crate::endpoints::v1::projects::project_id::get::view::GetProjectResultView;
use crate::endpoints::v1::projects::project_id::ProjectPathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum GetProjectError {
    NotFound,
    DatabaseError,
}

impl std::fmt::Display for GetProjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetProjectError::NotFound => write!(f, "Unknown project."),
            GetProjectError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for GetProjectError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetProjectError::NotFound => StatusCode::NOT_FOUND,
            GetProjectError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_get_project(
    state: web::Data<AppState>,
    user_id: u64,
    project_id: u64,
) -> Result<GetProjectResultView, GetProjectError> {
    let smart_db = state.get_smart_db();
    let projects: Vec<ProjectView> = smart_db
        .fetch_all(&GetVisibleProjectQueryView::new(project_id, user_id))
        .await
        .map_err(|_| GetProjectError::DatabaseError)?;
    // Un projet invisible pour l'appelant est traité comme inexistant.
    let project = projects
        .into_iter()
        .next()
        .ok_or(GetProjectError::NotFound)?;

    let tasks_view = GetProjectTasksQueryView::new(project_id);
    let users_view = GetProjectUsersQueryView::new(project_id);
    let (tasks, users) = futures_util::try_join!(
        smart_db.fetch_all::<Task, _>(&tasks_view),
        smart_db.fetch_all::<ProjectMemberRow, _>(&users_view),
    )
    .map_err(|_| GetProjectError::DatabaseError)?;

    Ok(GetProjectResultView {
        project: project.into(),
        tasks: tasks.into_iter().map(Into::into).collect(),
        users: users.into_iter().map(Into::into).collect(),
    })
}

#[utoipa::path(
    get,
    params(
        ProjectPathParams,
    ),
    path = "",
    summary = "Consulter un projet",
    description = "Renvoie un projet avec **toutes ses tâches et tous ses membres** en une seule \
                   requête : c'est l'appel que fait le front pour afficher un tableau de projet, \
                   plutôt que d'enchaîner `/tasks/` et `/users/`.\n\n\
                   Il suffit d'être membre du projet. Un projet auquel l'appelant n'a pas accès \
                   répond `404`, pas `403`.",
    responses(
        (
            status = 200,
            description = "Projet, ses tâches et ses membres.",
            body = GetProjectResultView,
            example = json!({
                "project": { "id": 12, "name": "Réfection de la place du marché", "description": "Travaux de voirie 2026", "status": "Active" },
                "tasks": [
                    {
                        "id": 77,
                        "title": "Consulter les riverains",
                        "description": "",
                        "status": "InProgress",
                        "priority": "High",
                        "due_date": "2026-10-15T00:00:00Z",
                        "assigned_to": 42,
                        "fields": []
                    }
                ],
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
    tag = "Projects",
)]
#[get("/")]
pub async fn get_project(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<ProjectPathParams>,
) -> Result<impl Responder, GetProjectError> {
    let result = trigger_get_project(state, auth_user.id, params.project_id()).await?;
    Ok(HttpResponse::Ok().json(result))
}
