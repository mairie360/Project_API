use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

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
                QueryParam::I32(owner_id as i32),
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
        self.params[2].as_i32() as u64
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
