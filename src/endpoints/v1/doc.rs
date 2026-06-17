use utoipa::OpenApi;
use crate::endpoints::v1::projects::doc::ProjectDoc;

#[derive(OpenApi)]
#[openapi(nest(
    (path = "/projects", api = ProjectDoc),
))]
pub struct V1Doc;
