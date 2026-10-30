pub mod doc;
pub mod get;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(actix_web::web::scope("/tasks").service(get::endpoint::get_tasks));
}
