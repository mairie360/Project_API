use utoipa::OpenApi;

use crate::endpoints::v1::projects::get::endpoint::__path_get_projects;
use crate::endpoints::v1::projects::get::view::GetProjectsResultView;
use crate::endpoints::v1::projects::post::endpoint::__path_create_project;
use crate::endpoints::v1::projects::post::view::{CreateProjectResultView, CreateProjectView};
use crate::endpoints::v1::projects::project_id::doc::IdDoc;
use crate::endpoints::v1::projects::templates::doc::TemplatesDoc;

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/", api = Doc),
        (path = "/{project_id}", api = IdDoc),
        (path = "/templates", api = TemplatesDoc),
    )
)]
pub struct ProjectDoc;

#[derive(OpenApi)]
#[openapi(
    paths(get_projects, create_project),
    components(schemas(GetProjectsResultView, CreateProjectResultView, CreateProjectView))
)]
struct Doc;
