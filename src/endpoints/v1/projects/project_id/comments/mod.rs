//! Comments of a project (MAIR-1089): posted by anyone who sees the project, read through its
//! activity (`GET …/activity/`).
pub mod doc;
pub mod endpoint;
pub mod view;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(actix_web::web::scope("/comments").service(endpoint::add_project_comment));
}
