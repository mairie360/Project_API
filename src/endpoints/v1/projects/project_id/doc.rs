use crate::endpoints::v1::projects::project_id::close::endpoint::__path_close_project;
use crate::endpoints::v1::projects::project_id::delete::endpoint::__path_delete_project;
use crate::endpoints::v1::projects::project_id::get::endpoint::__path_get_project;
use crate::endpoints::v1::projects::project_id::get::view::GetProjectResultView;
use crate::endpoints::v1::projects::project_id::tasks::doc::TasksDoc;
use crate::endpoints::v1::projects::project_id::users::doc::UsersDoc;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/", api = Doc),
        (path = "/tasks", api = TasksDoc),
        (path = "/users", api = UsersDoc),
    )
)]
pub struct IdDoc;

#[derive(OpenApi)]
#[openapi(
    paths(close_project, delete_project, get_project),
    components(schemas(GetProjectResultView))
)]
struct Doc;
