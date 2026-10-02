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
                       row_number() OVER (ORDER BY u.last_name, u.first_name, u.id) AS rn \
                FROM project_members pm JOIN users u ON u.id = pm.user_id \
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
    /// Identifiant Core API du membre.
    pub id: i32,
    /// Prénom et nom du membre, ou `null` si le nom n'a pas pu être résolu.
    pub name: Option<String>,
}
