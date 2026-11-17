use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;
use utoipa::ToSchema;

use crate::database::project::group::view::{GroupRow, ListLinkableGroupsQueryView};
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
         example = json!({ "groups": [ { "id": 5, "name": "Espaces verts" }, { "id": 3, "name": "Voirie" } ] })),
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
