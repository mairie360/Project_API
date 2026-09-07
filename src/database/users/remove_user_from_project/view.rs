use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RemoveUserFromProjectQueryView {
    params: Vec<QueryParam>,
}

impl RemoveUserFromProjectQueryView {
    pub fn new(project_id: u64, user_id: u64) -> Self {
        Self {
            params: vec![
                QueryParam::I32(project_id as i32),
                QueryParam::I32(user_id as i32),
            ],
        }
    }

    pub fn project_id(&self) -> u64 {
        self.params[0].as_i32() as u64
    }

    pub fn user_id(&self) -> u64 {
        self.params[1].as_i32() as u64
    }
}

impl ApiRequestDto for RemoveUserFromProjectQueryView {
    fn query_sql(&self) -> &'static str {
        "DELETE FROM project_members WHERE project_id = $1 AND user_id = $2"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
