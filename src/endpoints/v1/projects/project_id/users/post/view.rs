use actix_web::web;
use utoipa::ToSchema;

use crate::endpoints::v1::projects::project_id::users::post::endpoint::AddUserToProjectError;

/// Utilisateur à rattacher au projet du chemin.
#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct AddUserToProjectView {
    /// Identifiant Core API de l'utilisateur à rattacher, tel que le renvoie
    /// `GET /api/v1/user/` de Core API.
    #[schema(example = 42)]
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
