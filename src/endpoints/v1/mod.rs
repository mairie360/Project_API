pub mod doc;
pub mod projects;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(actix_web::web::scope("/v1").configure(projects::config));
}
