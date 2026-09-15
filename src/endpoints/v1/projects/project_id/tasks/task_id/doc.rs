use utoipa::OpenApi;

use crate::database::tasks::collaboration::view::{
    CollaborationAuthor, TaskComment, TaskHistoryEntry,
};
use crate::endpoints::v1::projects::project_id::tasks::task_id::collaboration::endpoint::__path_get_task_collaboration;
use crate::endpoints::v1::projects::project_id::tasks::task_id::collaboration::view::TaskCollaborationView;
use crate::endpoints::v1::projects::project_id::tasks::task_id::comments::endpoint::__path_add_task_comment;
use crate::endpoints::v1::projects::project_id::tasks::task_id::comments::view::AddTaskCommentView;
use crate::endpoints::v1::projects::project_id::tasks::task_id::delete::endpoint::__path_delete_task;
use crate::endpoints::v1::projects::project_id::tasks::task_id::history::endpoint::__path_append_task_history;
use crate::endpoints::v1::projects::project_id::tasks::task_id::history::view::AppendTaskHistoryView;
use crate::endpoints::v1::projects::project_id::tasks::task_id::patch::endpoint::__path_patch_task;
use crate::endpoints::v1::projects::project_id::tasks::task_id::patch::view::PatchTaskView;

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/", api = Doc),
    )
)]
pub struct TaskIdDoc;

#[derive(OpenApi)]
#[openapi(
    paths(
        delete_task,
        patch_task,
        get_task_collaboration,
        add_task_comment,
        append_task_history
    ),
    components(schemas(
        PatchTaskView,
        TaskCollaborationView,
        TaskComment,
        TaskHistoryEntry,
        CollaborationAuthor,
        AddTaskCommentView,
        AppendTaskHistoryView
    ))
)]
struct Doc;
