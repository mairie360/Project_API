use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// `SELECT 1`: a round-trip to Postgres, used by the readiness probe and the startup check.
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct PingQueryView {
    params: Vec<QueryParam>,
}

impl ApiRequestDto for PingQueryView {
    fn query_sql(&self) -> &'static str {
        "SELECT 1"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
