use utoipa::OpenApi;

use crate::endpoints::v1::projects::bulk_delete::endpoint::__path_bulk_delete_projects;
use crate::endpoints::v1::projects::bulk_delete::view::{
    BulkDeleteProjectsResultView, BulkDeleteProjectsView, BulkDeleteResultItem, BulkDeleteStatus,
};

#[derive(OpenApi)]
#[openapi(nest((path = "/", api = Doc)))]
pub struct BulkDeleteDoc;

#[derive(OpenApi)]
#[openapi(
    paths(bulk_delete_projects),
    components(schemas(
        BulkDeleteProjectsView,
        BulkDeleteStatus,
        BulkDeleteResultItem,
        BulkDeleteProjectsResultView
    ))
)]
struct Doc;
