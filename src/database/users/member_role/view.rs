use mairie360_api_lib::database::db_interface::{id_to_sql, ApiRequestDto, QueryParam};

/// Changes the role of a member of a project (MAIR-1088). Read with `fetch_scalar::<bool, _>`:
/// `false` when the user is not a member of the project.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SetMemberRoleQueryView {
    params: Vec<QueryParam>,
}

impl SetMemberRoleQueryView {
    /// `role` is `manager`, `contributor` or `reader` (`project_member_role`).
    pub fn new(project_id: u64, user_id: u64, role: &str) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(project_id)),
                QueryParam::I32(id_to_sql(user_id)),
                QueryParam::Text(role.to_string()),
            ],
        }
    }
}

impl ApiRequestDto for SetMemberRoleQueryView {
    fn query_sql(&self) -> &'static str {
        "WITH changed AS ( \
            UPDATE project_members SET role = $3::project_member_role \
            WHERE project_id = $1 AND user_id = $2 RETURNING user_id \
         ) SELECT EXISTS (SELECT 1 FROM changed)"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }

    // The projects list shows the role of its first members.
    fn invalidated_read_caches(&self) -> &'static [&'static str] {
        crate::database::project::PROJECTS_READ_CACHES
    }
}

/// `true` when the user owns the project (MAIR-1088): the owner always reads `manager`, so their
/// role cannot be changed and they cannot be removed from the members. Read with
/// `fetch_scalar::<bool, _>`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IsProjectOwnerQueryView {
    params: Vec<QueryParam>,
}

impl IsProjectOwnerQueryView {
    pub fn new(project_id: u64, user_id: u64) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(project_id)),
                QueryParam::I32(id_to_sql(user_id)),
            ],
        }
    }
}

impl ApiRequestDto for IsProjectOwnerQueryView {
    fn query_sql(&self) -> &'static str {
        "SELECT EXISTS (SELECT 1 FROM projects WHERE id = $1 AND owner_id = $2)"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
