use utoipa::ToSchema;

use crate::database::users::get_project_users::view::ProjectMemberRow;

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct User {
    /// Identifiant Core API du membre.
    #[schema(example = 42)]
    pub id: u64,
    /// Prénom et nom du membre, ou `null` si le nom n'a pas pu être résolu côté Core API.
    #[schema(example = "Jean Dupont")]
    pub name: Option<String>,
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetProjectUsersResultView {
    /// Membres rattachés au projet.
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
