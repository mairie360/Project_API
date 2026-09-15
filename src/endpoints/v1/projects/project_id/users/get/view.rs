use utoipa::ToSchema;

use crate::database::users::get_project_users::view::ProjectMemberRow;

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct User {
    pub id: u64,
    /// Prénom et nom du membre.
    pub name: Option<String>,
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetProjectUsersResultView {
    pub users: Vec<User>,
}

impl From<ProjectMemberRow> for User {
    fn from(row: ProjectMemberRow) -> Self {
        Self {
            id: row.id as u64,
            name: row.name,
        }
    }
}
