use utoipa::OpenApi;

use crate::endpoints::v1::projects::project_id::users::user_id::delete::endpoint::__path_remove_user_from_project;
use crate::endpoints::v1::projects::project_id::users::user_id::patch::endpoint::__path_update_member_role;
use crate::endpoints::v1::projects::project_id::users::user_id::patch::view::UpdateMemberRoleView;

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/", api = Doc),
    )
)]
pub struct UserIdDoc;

#[derive(OpenApi)]
#[openapi(
    paths(remove_user_from_project, update_member_role),
    components(schemas(UpdateMemberRoleView))
)]
struct Doc;
