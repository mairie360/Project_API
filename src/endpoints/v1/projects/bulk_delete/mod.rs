//! Bulk deletion of projects (MAIR-1089): each project is checked and deleted on its own.
pub mod doc;
pub mod endpoint;
pub mod view;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(actix_web::web::scope("/bulk-delete").service(endpoint::bulk_delete_projects));
}
