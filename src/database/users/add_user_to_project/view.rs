use mairie360_api_lib::database::db_interface::{
    id_from_sql, id_to_sql, ApiRequestDto, QueryParam,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AddUserToProjectQueryView {
    params: Vec<QueryParam>,
}

impl AddUserToProjectQueryView {
    pub fn new(project_id: u64, user_id: u64) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(project_id)),
                QueryParam::I32(id_to_sql(user_id)),
                QueryParam::Text(String::new()),
            ],
        }
    }

    /// Adds the member with this role (`manager`, `contributor` or `reader`, MAIR-1088) instead of
    /// the default `contributor`.
    #[must_use]
    pub fn with_role(mut self, role: Option<&str>) -> Self {
        self.params[2] = QueryParam::Text(role.unwrap_or_default().to_string());
        self
    }

    pub fn project_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }

    pub fn user_id(&self) -> u64 {
        id_from_sql(self.params[1].as_i32())
    }
}

impl ApiRequestDto for AddUserToProjectQueryView {
    fn query_sql(&self) -> &'static str {
        "INSERT INTO project_members (project_id, user_id, role) \
         VALUES ($1, $2, COALESCE(NULLIF($3, '')::project_member_role, 'contributor'))"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }

    fn invalidated_read_caches(&self) -> &'static [&'static str] {
        crate::database::project::PROJECTS_READ_CACHES
    }
}
