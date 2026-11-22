use utoipa::OpenApi;

use crate::endpoints::v1::projects::project_id::comments::endpoint::__path_add_project_comment;
use crate::endpoints::v1::projects::project_id::comments::view::AddProjectCommentView;

#[derive(OpenApi)]
#[openapi(nest((path = "/", api = Doc)))]
pub struct CommentsDoc;

#[derive(OpenApi)]
#[openapi(paths(add_project_comment), components(schemas(AddProjectCommentView)))]
struct Doc;
