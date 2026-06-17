pub mod doc;
pub mod get;
pub mod post;
pub mod task_id;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/tasks")
            .service(get::endpoint::get_project_tasks)
            .service(post::endpoint::create_task)
            .configure(task_id::config),
    );
}
