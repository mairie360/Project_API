use mairie360_api_lib::database::db_interface::{
    id_from_sql, id_to_sql, ApiRequestDto, QueryParam,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateProjectQueryView {
    params: Vec<QueryParam>,
}

impl CreateProjectQueryView {
    pub fn new(title: &str, description: Option<&str>, owner_id: u64) -> Self {
        Self {
            params: vec![
                QueryParam::Text(title.to_string()),
                QueryParam::Text(description.unwrap_or_default().to_string()),
                QueryParam::I32(id_to_sql(owner_id)),
            ],
        }
    }

    pub fn title(&self) -> &str {
        self.params[0].as_text()
    }

    pub fn description(&self) -> &str {
        self.params[1].as_text()
    }

    pub fn owner_id(&self) -> u64 {
        id_from_sql(self.params[2].as_i32())
    }
}

impl ApiRequestDto for CreateProjectQueryView {
    fn query_sql(&self) -> &'static str {
        "INSERT INTO projects (title, description, owner_id) \
         VALUES ($1, NULLIF($2, ''), $3) RETURNING id"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
