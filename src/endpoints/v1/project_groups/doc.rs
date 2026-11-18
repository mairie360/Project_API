use utoipa::OpenApi;

use crate::endpoints::v1::project_groups::endpoint::{
    __path_get_project_groups, __path_get_projects_without_group, GetProjectGroupsResultView,
    GetProjectsWithoutGroupResultView, ProjectWithoutGroupView,
};
use crate::endpoints::v1::projects::get::view::ProjectGroupView;

#[derive(OpenApi)]
#[openapi(
    paths(get_project_groups, get_projects_without_group),
    components(schemas(
        GetProjectGroupsResultView,
        ProjectGroupView,
        GetProjectsWithoutGroupResultView,
        ProjectWithoutGroupView
    ))
)]
pub struct ProjectGroupsDoc;
