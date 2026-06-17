pub mod doc;
pub mod get;
pub mod post;
pub mod project_id;
pub mod templates;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/projects")
            .service(get::endpoint::get_projects)
            .service(post::endpoint::create_project)
            .configure(project_id::config)
            .configure(templates::config),
    );
}
