use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetProjectUsersQueryView {
    params: Vec<QueryParam>,
}

impl GetProjectUsersQueryView {
    pub fn new(project_id: u64) -> Self {
        Self {
            params: vec![QueryParam::I32(project_id as i32)],
        }
    }

    pub fn project_id(&self) -> u64 {
        self.params[0].as_i32() as u64
    }
}

impl ApiRequestDto for GetProjectUsersQueryView {
    fn query_sql(&self) -> &'static str {
        "SELECT to_jsonb(user_id) FROM project_members WHERE project_id = $1"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
