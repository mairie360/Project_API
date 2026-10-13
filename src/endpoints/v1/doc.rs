use crate::endpoints::v1::projects::doc::ProjectDoc;
use crate::endpoints::v1::tasks::doc::TasksDoc;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(nest(
    (path = "/projects", api = ProjectDoc),
    (path = "/tasks", api = TasksDoc),
))]
pub struct V1Doc;
