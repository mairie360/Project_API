use mairie360_api_lib::database::db_interface::{
    id_from_sql, id_to_sql, ApiRequestDto, QueryParam,
};

/// One page of the members of a project, by name. Read with
/// `fetch_one::<PagedRows<ProjectMemberRow>, _>`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetProjectUsersQueryView {
    params: Vec<QueryParam>,
}

impl GetProjectUsersQueryView {
    pub fn new(project_id: u64, limit: u32, offset: u32) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(project_id)),
                QueryParam::I64(i64::from(limit)),
                QueryParam::I64(i64::from(offset)),
            ],
        }
    }

    pub fn project_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }
}

impl ApiRequestDto for GetProjectUsersQueryView {
    fn query_sql(&self) -> &'static str {
        concat!(
            crate::paged_rows_sql!("$2", "$3"),
            " FROM ( \
                SELECT u.id, NULLIF(concat_ws(' ', u.first_name, u.last_name), '') AS name, \
                       ",
            crate::member_role_sql!(),
            " AS role, \
                       row_number() OVER (ORDER BY u.last_name, u.first_name, u.id) AS rn \
                FROM project_members pm JOIN users u ON u.id = pm.user_id \
                JOIN projects p ON p.id = pm.project_id \
                WHERE pm.project_id = $1 \
             ) t"
        )
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProjectMemberRow {
    /// Core API id of the member.
    pub id: i32,
    /// First and last name of the member, or `null` if the name could not be resolved.
    pub name: Option<String>,
    /// Role of the member in the project (MAIR-1088): `manager`, `contributor` or `reader`; the
    /// owner of the project always reads `manager`. Defaulted: a page cached before MAIR-1088 has
    /// none.
    #[serde(default = "default_member_role")]
    pub role: String,
}

/// Role of a member added without one (Database `project_members.role` default).
pub const DEFAULT_MEMBER_ROLE: &str = "contributor";

fn default_member_role() -> String {
    DEFAULT_MEMBER_ROLE.to_string()
}

/// SQL of the role a member `pm` reads with (MAIR-1088): `manager` for the owner of its project
/// `p`, the stored `project_members.role` otherwise. Needs `pm` and `p` in scope.
#[macro_export]
macro_rules! member_role_sql {
    () => {
        "(CASE WHEN pm.user_id = p.owner_id THEN 'manager' ELSE pm.role::text END)"
    };
}
