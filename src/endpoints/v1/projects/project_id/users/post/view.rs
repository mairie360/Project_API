use utoipa::ToSchema;

/// User to add to the project of the path.
#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct AddUserToProjectView {
    /// Core API id of the user to add, as returned by
    /// `GET /api/v1/user/` of Core API.
    #[schema(example = 42)]
    pub user_id: u64,
}
