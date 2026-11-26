use crate::endpoints::v1::project_groups::doc::ProjectGroupsDoc;
use crate::endpoints::v1::project_templates::doc::TemplatesDoc;
use crate::endpoints::v1::projects::doc::ProjectDoc;
use crate::endpoints::v1::search::doc::SearchDoc;
use crate::endpoints::v1::tasks::doc::TasksDoc;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(nest(
    (path = "/projects", api = ProjectDoc),
    (path = "/project-templates", api = TemplatesDoc),
    (path = "/project-groups", api = ProjectGroupsDoc),
    (path = "/tasks", api = TasksDoc),
    (path = "/search", api = SearchDoc),
))]
pub struct V1Doc;
