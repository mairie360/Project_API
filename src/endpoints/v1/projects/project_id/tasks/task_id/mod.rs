pub mod collaboration;
pub mod comments;
pub mod delete;
pub mod doc;
pub mod history;
pub mod patch;

#[derive(serde::Deserialize, utoipa::IntoParams, utoipa::ToSchema)]
#[into_params(parameter_in = Path)]
pub struct TaskPathParams {
    project_id: u64,
    task_id: u64,
}

impl TaskPathParams {
    pub fn new(project_id: u64, task_id: u64) -> Self {
        Self {
            project_id,
            task_id,
        }
    }

    pub fn project_id(&self) -> u64 {
        self.project_id
    }

    pub fn task_id(&self) -> u64 {
        self.task_id
    }
}

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/{task_id}")
            .service(delete::endpoint::delete_task)
            .service(patch::endpoint::patch_task)
            .service(collaboration::endpoint::get_task_collaboration)
            .service(comments::endpoint::add_task_comment)
            .service(history::endpoint::append_task_history),
    );
}
