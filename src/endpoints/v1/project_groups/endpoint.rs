use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;
use utoipa::ToSchema;

use crate::database::project::group::view::{
    GroupRow, ListLinkableGroupsQueryView, ListProjectsWithoutGroupQueryView,
    ProjectWithoutGroupRow,
};
use crate::endpoints::db_error::log_db_error;
use crate::endpoints::v1::projects::get::view::ProjectGroupView;

/// Groups the caller may link to a project, by name.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetProjectGroupsResultView {
    /// The caller's groups, or every group for Admin and Maire. Empty when the caller belongs to
    /// no group.
    pub groups: Vec<ProjectGroupView>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GetProjectGroupsError {
    DatabaseError,
}

impl std::fmt::Display for GetProjectGroupsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "An error occurred while accessing the database.")
    }
}

impl ResponseError for GetProjectGroupsError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

#[utoipa::path(
    get,
    path = "/",
    summary = "List the groups a project can be linked to",
    description = "Returns the Core API groups the caller may send as `group_id` when creating \
                   (`POST /api/v1/projects/`) or updating (`PATCH /api/v1/projects/{project_id}/`) \
                   a project, ordered by name: the groups the caller is a member of, or every \
                   group for the Admin and Maire roles. Every current and future member of the \
                   group linked to a project sees that project. Any signed-in user may call it; \
                   the list is empty for a caller without group.",
    responses(
        (status = 200, description = "The groups the caller may link.", body = GetProjectGroupsResultView,
         example = json!({ "groups": [ { "id": 5, "name": "Green spaces" }, { "id": 3, "name": "Roads" } ] })),
        (status = 401, description = "Missing `Authorization` header, invalid or expired JWT, or revoked session.",
         body = String, content_type = "text/plain", example = json!("Jeton expiré")),
        (status = 500, description = "Database error.", body = String, content_type = "text/plain",
         example = json!("An error occurred while accessing the database.")),
    ),
    security(("jwt" = [])),
    tag = "Projects",
)]
#[get("/")]
pub async fn get_project_groups(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl Responder, GetProjectGroupsError> {
    let rows: Vec<GroupRow> = state
        .get_smart_db()
        .fetch_one(&ListLinkableGroupsQueryView::new(auth_user.id))
        .await
        .map_err(|e| {
            log_db_error("project-groups/get", &e);
            GetProjectGroupsError::DatabaseError
        })?;
    Ok(HttpResponse::Ok().json(GetProjectGroupsResultView {
        groups: rows.iter().map(Into::into).collect(),
    }))
}

/// A project linked to no group.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct ProjectWithoutGroupView {
    #[schema(example = 9)]
    pub id: i32,
    #[schema(example = "LED street lighting")]
    pub name: String,
    /// Creation date (ISO 8601), `null` when unknown.
    #[schema(example = "2026-03-03T09:12:00+00:00")]
    pub created_at: Option<String>,
}

impl From<ProjectWithoutGroupRow> for ProjectWithoutGroupView {
    fn from(row: ProjectWithoutGroupRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            created_at: row.created_at,
        }
    }
}

/// The projects linked to no group.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetProjectsWithoutGroupResultView {
    /// Newest first; empty when every project has a group.
    pub projects: Vec<ProjectWithoutGroupView>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GetProjectsWithoutGroupError {
    Forbidden,
    DatabaseError,
}

impl std::fmt::Display for GetProjectsWithoutGroupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetProjectsWithoutGroupError::Forbidden => write!(f, "Forbidden."),
            GetProjectsWithoutGroupError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for GetProjectsWithoutGroupError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetProjectsWithoutGroupError::Forbidden => StatusCode::FORBIDDEN,
            GetProjectsWithoutGroupError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

#[utoipa::path(
    get,
    path = "/projects-without-group/",
    summary = "List the projects linked to no group",
    description = "Every project must have a group, but a project created before groups were \
                   required, or \
                   whose group was deleted in Core API, has none until a manager picks one with \
                   `PATCH /api/v1/projects/{project_id}/`. Lists those projects, newest first, \
                   for the administration page. Restricted to the Admin and Maire roles (`403` \
                   otherwise).",
    responses(
        (status = 200, description = "The projects linked to no group; empty when every project has one.",
         body = GetProjectsWithoutGroupResultView,
         example = json!({ "projects": [ { "id": 9, "name": "LED street lighting", "created_at": "2026-03-03T09:12:00+00:00" } ] })),
        (status = 401, description = "Missing `Authorization` header, invalid or expired JWT, or revoked session.",
         body = String, content_type = "text/plain", example = json!("Jeton expiré")),
        (status = 403, description = "The caller is neither Admin nor Maire.",
         body = String, content_type = "text/plain", example = json!("Forbidden.")),
        (status = 500, description = "Database error.", body = String, content_type = "text/plain",
         example = json!("An error occurred while accessing the database.")),
    ),
    security(("jwt" = [])),
    tag = "Projects",
)]
#[get("/projects-without-group/")]
pub async fn get_projects_without_group(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl Responder, GetProjectsWithoutGroupError> {
    let rows: Option<Vec<ProjectWithoutGroupRow>> = state
        .get_smart_db()
        .fetch_one(&ListProjectsWithoutGroupQueryView::new(auth_user.id))
        .await
        .map_err(|e| {
            log_db_error("project-groups/projects-without-group", &e);
            GetProjectsWithoutGroupError::DatabaseError
        })?;
    let rows = rows.ok_or(GetProjectsWithoutGroupError::Forbidden)?;
    Ok(HttpResponse::Ok().json(GetProjectsWithoutGroupResultView {
        projects: rows.into_iter().map(Into::into).collect(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mair_683_a_database_error_answers_500_without_its_detail() {
        let error = GetProjectGroupsError::DatabaseError;
        assert_eq!(error.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(
            error.to_string(),
            "An error occurred while accessing the database."
        );
        assert_eq!(
            error.error_response().status(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}
