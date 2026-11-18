//! Groups a caller may link to a project (`POST` / `PATCH /api/v1/projects/…` `group_id`).
pub mod doc;
pub mod endpoint;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/project-groups")
            .service(endpoint::get_projects_without_group)
            .service(endpoint::get_project_groups),
    );
}
