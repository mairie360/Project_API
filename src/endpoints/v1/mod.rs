pub mod doc;
pub mod project_templates;
pub mod projects;
pub mod tasks;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/v1")
            .configure(projects::config)
            .configure(project_templates::config)
            .configure(tasks::config),
    );
}
