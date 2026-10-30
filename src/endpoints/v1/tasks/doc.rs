use utoipa::OpenApi;

use crate::endpoints::v1::tasks::get::endpoint::__path_get_tasks;
use crate::endpoints::v1::tasks::get::view::{GetTasksListResultView, TaskListItemView};

#[derive(OpenApi)]
#[openapi(nest((path = "/", api = Doc)))]
pub struct TasksDoc;

#[derive(OpenApi)]
#[openapi(
    paths(get_tasks),
    components(schemas(GetTasksListResultView, TaskListItemView))
)]
struct Doc;
