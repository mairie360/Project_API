use actix_web::http::StatusCode;
use actix_web::{patch, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::smart_db::SmartTransaction;
use mairie360_api_lib::state::AppState;

use crate::database::users::member_role::view::{IsProjectOwnerQueryView, SetMemberRoleQueryView};
use crate::endpoints::db_error::log_db_error;
use crate::endpoints::v1::projects::access::{begin_write, commit, AccessDenied, Requirement};
use crate::endpoints::v1::projects::project_id::users::user_id::patch::view::UpdateMemberRoleView;
use crate::endpoints::v1::projects::project_id::users::user_id::ProjectUserPathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum UpdateMemberRoleError {
    Forbidden,
    /// Unknown or invisible project.
    NotFound,
    /// The user is not a member of the project.
    UnknownMember,
    /// The user owns the project, who always reads `manager`.
    Owner,
    DatabaseError,
}

impl std::fmt::Display for UpdateMemberRoleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdateMemberRoleError::Forbidden => write!(f, "Forbidden."),
            UpdateMemberRoleError::NotFound => write!(f, "Not found."),
            UpdateMemberRoleError::UnknownMember => {
                write!(f, "The user is not a member of this project.")
            }
            UpdateMemberRoleError::Owner => {
                write!(f, "The owner of the project is always its manager.")
            }
            UpdateMemberRoleError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for UpdateMemberRoleError {
    fn status_code(&self) -> StatusCode {
        match self {
            UpdateMemberRoleError::Forbidden => StatusCode::FORBIDDEN,
            UpdateMemberRoleError::NotFound | UpdateMemberRoleError::UnknownMember => {
                StatusCode::NOT_FOUND
            }
            UpdateMemberRoleError::Owner => StatusCode::CONFLICT,
            UpdateMemberRoleError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

impl From<AccessDenied> for UpdateMemberRoleError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => UpdateMemberRoleError::NotFound,
            AccessDenied::Forbidden => UpdateMemberRoleError::Forbidden,
            AccessDenied::DatabaseError => UpdateMemberRoleError::DatabaseError,
        }
    }
}

async fn trigger_update_member_role(
    tx: &mut SmartTransaction,
    project_id: u64,
    user_id: u64,
    view: &UpdateMemberRoleView,
) -> Result<(), UpdateMemberRoleError> {
    const OPERATION: &str = "projects/project_id/users/user_id/patch";
    let owner: bool = tx
        .fetch_scalar(&IsProjectOwnerQueryView::new(project_id, user_id))
        .await
        .map_err(|e| {
            log_db_error(OPERATION, &e);
            UpdateMemberRoleError::DatabaseError
        })?;
    if owner {
        return Err(UpdateMemberRoleError::Owner);
    }
    let changed: bool = tx
        .fetch_scalar(&SetMemberRoleQueryView::new(
            project_id,
            user_id,
            view.role.stored(),
        ))
        .await
        .map_err(|e| {
            log_db_error(OPERATION, &e);
            UpdateMemberRoleError::DatabaseError
        })?;
    if changed {
        Ok(())
    } else {
        Err(UpdateMemberRoleError::UnknownMember)
    }
}

#[utoipa::path(
    patch,
    path = "",
    summary = "Change the role of a member of a project",
    description = "Sets the role of a member of the project (MAIR-1088): `manager`, `contributor` \
                   or `reader`. Reserved to the managers of the project. The owner of the project \
                   always reads `manager` and cannot be changed (`409`); a user who is not a \
                   member answers `404`. The response has an empty body.",
    request_body(
        content = UpdateMemberRoleView,
        description = "The new role.",
        example = json!({ "role": "reader" })
    ),
    responses(
        (status = 204, description = "Role changed. Empty body."),
        (status = 400, description = "A URL segment is not an integer, malformed JSON body, missing `role`, or `role` not one of `manager`, `contributor`, `reader`.",
         body = String, content_type = "text/plain", example = json!("Json deserialize error: unknown variant `admin`")),
        (status = 401, description = "Missing `Authorization` header, invalid or expired JWT, or revoked session.",
         body = String, content_type = "text/plain", example = json!("Jeton expiré")),
        (status = 403, description = "Project visible to the caller, but insufficient rights for this operation.",
         body = String, content_type = "text/plain", example = json!("Forbidden.")),
        (status = 404, description = "Unknown or invisible project (`Not found.`), or user who is not a member of it (`The user is not a member of this project.`).",
         body = String, content_type = "text/plain", example = json!("The user is not a member of this project.")),
        (status = 409, description = "The user owns the project, who always reads `manager`.",
         body = String, content_type = "text/plain", example = json!("The owner of the project is always its manager.")),
        (status = 500, description = "Database error.", body = String, content_type = "text/plain",
         example = json!("An error occurred while accessing the database.")),
    ),
    params(ProjectUserPathParams),
    security(("jwt" = [])),
    tag = "Users",
)]
#[patch("/")]
pub async fn update_member_role(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<ProjectUserPathParams>,
    view: web::Json<UpdateMemberRoleView>,
) -> Result<impl Responder, UpdateMemberRoleError> {
    let (mut tx, _) = begin_write(
        &state,
        auth_user.id,
        params.project_id(),
        None,
        Requirement::ManageProject,
    )
    .await?;
    trigger_update_member_role(&mut tx, params.project_id(), params.user_id(), &view).await?;
    commit(tx, "projects/project_id/users/user_id/patch").await?;
    Ok(HttpResponse::NoContent().finish())
}
