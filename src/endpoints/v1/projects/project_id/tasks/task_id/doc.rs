use utoipa::OpenApi;

use crate::endpoints::v1::projects::project_id::tasks::task_id::delete::endpoint::__path_delete_task;

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/", api = Doc),
    )
)]
pub struct TaskIdDoc;

#[derive(OpenApi)]
// #[openapi(paths(delete_task, patch_task), components(schemas(PatchTaskView)))]
#[openapi(paths(delete_task), components(schemas()))]
struct Doc;
