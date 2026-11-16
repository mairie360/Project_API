use mairie360_api_lib::database::db_interface::id_from_sql;
use utoipa::ToSchema;

use crate::database::users::get_project_users::view::ProjectMemberRow;

/// Member of a project.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct User {
    /// Core API id of the member.
    #[schema(example = 42)]
    pub id: u64,
    /// First and last name of the member, or `null` if Core API could not resolve the name.
    #[schema(example = "Jean Dupont")]
    pub name: Option<String>,
}

/// One page of the members of a project.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetProjectUsersResultView {
    /// Members of the page, sorted by last name, first name, then id.
    pub users: Vec<User>,
    /// Number of members of the project, whatever the page.
    #[schema(example = 2)]
    pub total: u64,
}

impl From<ProjectMemberRow> for User {
    fn from(row: ProjectMemberRow) -> Self {
        Self {
            id: id_from_sql(row.id),
            name: row.name,
        }
    }
}
