use utoipa::OpenApi;

use crate::endpoints::v1::project_groups::endpoint::{
    __path_get_project_groups, GetProjectGroupsResultView,
};
use crate::endpoints::v1::projects::get::view::ProjectGroupView;

#[derive(OpenApi)]
#[openapi(
    paths(get_project_groups),
    components(schemas(GetProjectGroupsResultView, ProjectGroupView))
)]
pub struct ProjectGroupsDoc;
