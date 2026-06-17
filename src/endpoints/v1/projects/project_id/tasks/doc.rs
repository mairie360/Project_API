use crate::endpoints::v1::projects::project_id::tasks::get::endpoint::__path_get_project_tasks;
use crate::endpoints::v1::projects::project_id::tasks::get::view::GetTasksResultView;
use crate::endpoints::v1::projects::project_id::tasks::post::endpoint::__path_create_task;
use crate::endpoints::v1::projects::project_id::tasks::post::view::{
    CreateTaskResultView, CreateTaskView,
};
use crate::endpoints::v1::projects::project_id::tasks::task_id::doc::TaskIdDoc;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/", api = Doc),
        (path = "/{task_id}", api = TaskIdDoc),
    )
)]
pub struct TasksDoc;

#[derive(OpenApi)]
#[openapi(
    paths(get_project_tasks, create_task),
    components(schemas(GetTasksResultView, CreateTaskResultView, CreateTaskView))
)]
struct Doc;
