use utoipa::OpenApi;

use crate::endpoints::v1::projects::get::endpoint::__path_get_projects;
use crate::endpoints::v1::projects::get::view::{
    GetProjectsResultView, ProjectListItemView, ProjectPriorityCountsView, ProjectStatusCountsView,
    ProjectsSummaryView,
};
use crate::endpoints::v1::projects::post::endpoint::__path_create_project;
use crate::endpoints::v1::projects::post::view::{CreateProjectResultView, CreateProjectView};
use crate::endpoints::v1::projects::project_id::doc::IdDoc;

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/", api = Doc),
        (path = "/{project_id}", api = IdDoc),
    )
)]
pub struct ProjectDoc;

#[derive(OpenApi)]
#[openapi(
    paths(get_projects, create_project),
    components(schemas(
        GetProjectsResultView,
        ProjectListItemView,
        ProjectsSummaryView,
        ProjectStatusCountsView,
        ProjectPriorityCountsView,
        CreateProjectResultView,
        CreateProjectView
    ))
)]
struct Doc;
