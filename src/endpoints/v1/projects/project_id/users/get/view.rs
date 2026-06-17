use utoipa::ToSchema;

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct User {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetProjectUsersResultView {
    pub users: Vec<User>,
}
