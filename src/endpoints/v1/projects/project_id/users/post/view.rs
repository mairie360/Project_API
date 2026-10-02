use utoipa::ToSchema;

/// Utilisateur à rattacher au projet du chemin.
#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct AddUserToProjectView {
    /// Identifiant Core API de l'utilisateur à rattacher, tel que le renvoie
    /// `GET /api/v1/user/` de Core API.
    #[schema(example = 42)]
    pub user_id: u64,
}
