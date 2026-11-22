pub mod access;
pub mod bulk_delete;
pub mod doc;
pub mod get;
pub mod post;
pub mod project_id;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/projects")
            .service(get::endpoint::get_projects)
            .service(post::endpoint::create_project)
            // Before `/{project_id}`, which would take `bulk-delete` for an id.
            .configure(bulk_delete::config)
            .configure(project_id::config),
    );
}
