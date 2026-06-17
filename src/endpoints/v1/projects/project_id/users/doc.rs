use utoipa::OpenApi;

use crate::endpoints::v1::projects::project_id::users::get::endpoint::__path_get_project_users;
use crate::endpoints::v1::projects::project_id::users::get::view::GetProjectUsersResultView;
use crate::endpoints::v1::projects::project_id::users::post::endpoint::__path_add_user_to_project;
use crate::endpoints::v1::projects::project_id::users::post::view::AddUserToProjectView;
use crate::endpoints::v1::projects::project_id::users::user_id::doc::UserIdDoc;

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/", api = Doc),
        (path = "/{user_id}", api = UserIdDoc),
    )
)]
pub struct UsersDoc;

#[derive(OpenApi)]
#[openapi(
    paths(get_project_users, add_user_to_project),
    components(schemas(GetProjectUsersResultView, AddUserToProjectView))
)]
struct Doc;
