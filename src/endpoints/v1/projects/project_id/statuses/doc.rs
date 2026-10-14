use utoipa::OpenApi;

use crate::database::statuses::view::StatusCategory;
use crate::endpoints::v1::projects::project_id::statuses::endpoint::{
    __path_create_project_status, __path_delete_project_status, __path_get_project_statuses,
    __path_update_project_status,
};
use crate::endpoints::v1::projects::project_id::statuses::view::{
    CreateProjectStatusResultView, CreateProjectStatusView, GetProjectStatusesResultView,
    PatchProjectStatusView, ProjectStatusView,
};

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/", api = Doc),
        (path = "/{status_id}", api = StatusIdDoc),
    )
)]
pub struct StatusesDoc;

#[derive(OpenApi)]
#[openapi(
    paths(get_project_statuses, create_project_status),
    components(schemas(
        StatusCategory,
        ProjectStatusView,
        GetProjectStatusesResultView,
        CreateProjectStatusView,
        CreateProjectStatusResultView
    ))
)]
struct Doc;

#[derive(OpenApi)]
#[openapi(nest((path = "/", api = StatusIdOperationsDoc)))]
struct StatusIdDoc;

#[derive(OpenApi)]
#[openapi(
    paths(update_project_status, delete_project_status),
    components(schemas(PatchProjectStatusView))
)]
struct StatusIdOperationsDoc;
