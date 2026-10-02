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
            ],
        }
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
        "INSERT INTO project_members (project_id, user_id) VALUES ($1, $2)"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
