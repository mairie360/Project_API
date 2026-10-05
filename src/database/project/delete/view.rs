use mairie360_api_lib::database::db_interface::{
    id_from_sql, id_to_sql, ApiRequestDto, QueryParam,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeleteProjectQueryView {
    params: Vec<QueryParam>,
}

impl DeleteProjectQueryView {
    pub fn new(project_id: u64) -> Self {
        Self {
            params: vec![QueryParam::I32(id_to_sql(project_id))],
        }
    }

    pub fn project_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }
}

impl ApiRequestDto for DeleteProjectQueryView {
    fn query_sql(&self) -> &'static str {
        "DELETE FROM projects WHERE id = $1"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
