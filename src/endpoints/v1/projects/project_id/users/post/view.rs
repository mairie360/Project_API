use actix_web::web;
use utoipa::ToSchema;

use crate::endpoints::v1::projects::project_id::users::post::endpoint::AddUserToProjectError;

#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct AddUserToProjectView {
    pub user_id: u64,
}

impl TryFrom<web::Json<AddUserToProjectView>> for AddUserToProjectView {
    type Error = AddUserToProjectError;

    fn try_from(
        params: web::Json<AddUserToProjectView>,
    ) -> Result<AddUserToProjectView, Self::Error> {
        Ok(params.into_inner())
    }
}
