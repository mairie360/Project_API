use utoipa::OpenApi;

use crate::endpoints::v1::projects::project_id::archived_tasks::get::endpoint::__path_get_archived_tasks;

#[derive(OpenApi)]
#[openapi(nest((path = "/", api = Doc)))]
pub struct ArchivedTasksDoc;

#[derive(OpenApi)]
#[openapi(paths(get_archived_tasks))]
struct Doc;
