use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeleteTaskQueryView {
    params: Vec<QueryParam>,
}

impl DeleteTaskQueryView {
    pub fn new(task_id: u64) -> Self {
        Self {
            params: vec![QueryParam::I32(task_id as i32)],
        }
    }

    pub fn task_id(&self) -> u64 {
        self.params[0].as_i32() as u64
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
