use mairie360_api_lib::database::db_interface::id_from_sql;
use utoipa::ToSchema;

use crate::database::users::get_project_users::view::ProjectMemberRow;

/// Role of a member in a project (MAIR-1088). The owner of the project always reads `manager`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ProjectMemberRole {
    /// Manages the project with its owner.
    Manager,
    /// Works on the project. The role of a member added without one.
    Contributor,
    /// Follows the project.
    Reader,
}

impl ProjectMemberRole {
    /// The stored `project_member_role`.
    pub fn stored(self) -> &'static str {
        match self {
            ProjectMemberRole::Manager => "manager",
            ProjectMemberRole::Contributor => "contributor",
            ProjectMemberRole::Reader => "reader",
        }
    }

    /// The role of a stored value; an unknown one reads `contributor`, the default.
    pub fn from_stored(value: &str) -> Self {
        match value {
            "manager" => ProjectMemberRole::Manager,
            "reader" => ProjectMemberRole::Reader,
            _ => ProjectMemberRole::Contributor,
        }
    }
}

/// Member of a project.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct User {
    /// Core API id of the member.
    #[schema(example = 42)]
    pub id: u64,
    /// First and last name of the member, or `null` if Core API could not resolve the name.
    #[schema(example = "Jean Dupont")]
    pub name: Option<String>,
    /// Role of the member in the project (MAIR-1088): `manager`, `contributor` or `reader`. The
    /// owner of the project, when listed, always reads `manager`.
    pub role: ProjectMemberRole,
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
            role: ProjectMemberRole::from_stored(&row.role),
        }
    }
}
