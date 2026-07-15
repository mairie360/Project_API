pub mod close;
pub mod delete;
pub mod doc;
pub mod get;
pub mod tasks;
pub mod users;

#[derive(serde::Deserialize, utoipa::IntoParams, utoipa::ToSchema)]
#[into_params(parameter_in = Path)]
pub struct ProjectPathParams {
    project_id: u64,
}

impl ProjectPathParams {
    pub fn new(project_id: u64) -> Self {
        Self { project_id }
    }

    pub fn project_id(&self) -> u64 {
        self.project_id
    }
}

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/projects")
            // .service(get::endpoint::get_project)
            .service(close::endpoint::close_project)
            .service(delete::endpoint::delete_project)
            .configure(tasks::config)
            .configure(users::config),
    );
}
