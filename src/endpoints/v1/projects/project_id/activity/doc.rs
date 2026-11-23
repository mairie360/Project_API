use utoipa::OpenApi;

use crate::endpoints::v1::projects::project_id::activity::endpoint::__path_get_project_activity;
use crate::endpoints::v1::projects::project_id::activity::view::{
    ActivityItemView, ActivityKind, GetProjectActivityResultView,
};

#[derive(OpenApi)]
#[openapi(nest((path = "/", api = Doc)))]
pub struct ActivityDoc;

#[derive(OpenApi)]
#[openapi(
    paths(get_project_activity),
    components(schemas(ActivityKind, ActivityItemView, GetProjectActivityResultView))
)]
struct Doc;
