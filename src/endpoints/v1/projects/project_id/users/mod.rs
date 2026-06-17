pub mod doc;
pub mod get;
pub mod post;
pub mod user_id;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/users")
            .service(get::endpoint::get_project_users)
            .service(post::endpoint::add_user_to_project)
            .configure(user_id::config),
    );
}
