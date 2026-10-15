use crate::endpoints::v1::project_templates::doc::TemplatesDoc;
use crate::endpoints::v1::projects::doc::ProjectDoc;
use crate::endpoints::v1::tasks::doc::TasksDoc;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(nest(
    (path = "/projects", api = ProjectDoc),
    (path = "/project-templates", api = TemplatesDoc),
    (path = "/tasks", api = TasksDoc),
))]
pub struct V1Doc;
