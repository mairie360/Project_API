pub mod delete;
pub mod doc;

#[derive(serde::Deserialize, utoipa::IntoParams, utoipa::ToSchema)]
#[into_params(parameter_in = Path)]
pub struct ProjectUserPathParams {
    project_id: u64,
    user_id: u64,
}

impl ProjectUserPathParams {
    pub fn new(project_id: u64, user_id: u64) -> Self {
        Self {
            project_id,
            user_id,
        }
    }

    pub fn project_id(&self) -> u64 {
        self.project_id
    }

    pub fn user_id(&self) -> u64 {
        self.user_id
    }
}

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/{user_id}").service(delete::endpoint::remove_user_from_project),
    );
}
