pub mod activity;
pub mod archived_tasks;
pub mod close;
pub mod comments;
pub mod delete;
pub mod doc;
pub mod get;
pub mod milestones;
pub mod patch;
pub mod statuses;
pub mod tasks;
pub mod users;

/// Path parameters of the routes of a project.
#[derive(serde::Deserialize, utoipa::IntoParams, utoipa::ToSchema)]
#[into_params(parameter_in = Path)]
pub struct ProjectPathParams {
    /// Id of the project.
    #[param(example = 12)]
    #[schema(example = 12)]
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
        actix_web::web::scope("/{project_id}")
            .service(get::endpoint::get_project)
            .service(patch::endpoint::update_project)
            .service(close::endpoint::close_project)
            .service(delete::endpoint::delete_project)
            .configure(tasks::config)
            .configure(archived_tasks::config)
            .configure(statuses::config)
            .configure(milestones::config)
            .configure(activity::config)
            .configure(comments::config)
            .configure(users::config),
    );
}
