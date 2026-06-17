use utoipa::OpenApi;

use crate::endpoints::v1::projects::project_id::users::user_id::delete::endpoint::__path_remove_user_from_project;

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/", api = Doc),
    )
)]
pub struct UserIdDoc;

#[derive(OpenApi)]
#[openapi(paths(remove_user_from_project), components(schemas()))]
struct Doc;
