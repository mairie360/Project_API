use mairie360_api_lib::database::db_interface::{
    id_from_sql, id_to_sql, ApiRequestDto, QueryParam,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeleteTaskQueryView {
    params: Vec<QueryParam>,
}

impl DeleteTaskQueryView {
    pub fn new(task_id: u64) -> Self {
        Self {
            params: vec![QueryParam::I32(id_to_sql(task_id))],
        }
    }

    pub fn task_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }
}

impl ApiRequestDto for DeleteTaskQueryView {
    fn query_sql(&self) -> &'static str {
        "DELETE FROM tasks WHERE id = $1"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
