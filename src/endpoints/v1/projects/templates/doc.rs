use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/", api = Doc),
    )
)]
pub struct TemplatesDoc;

#[derive(OpenApi)]
#[openapi(paths(), components(schemas()))]
struct Doc;
