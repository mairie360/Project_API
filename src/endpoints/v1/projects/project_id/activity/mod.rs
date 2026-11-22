//! Activity feed of a project (MAIR-1089): its comments and the history of its tasks, read by
//! anyone who sees the project.
pub mod doc;
pub mod endpoint;
pub mod view;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(actix_web::web::scope("/activity").service(endpoint::get_project_activity));
}
