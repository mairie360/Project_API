use crate::endpoints::v1::projects::doc::ProjectDoc;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(nest(
    (path = "/projects", api = ProjectDoc),
))]
pub struct V1Doc;
