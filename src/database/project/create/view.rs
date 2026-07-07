use std::fmt::Display;

use mairie360_api_lib::database::db_interface::DatabaseQueryView;

pub struct CreateProjectQueryView {
    title: String,
    description: Option<String>,
    owner_id: u64,
}

impl CreateProjectQueryView {
    pub fn new(title: &str, description: Option<&str>, owner_id: u64) -> Self {
        Self {
            title: title.to_string(),
            description: description.map(|d| d.to_string()),
            owner_id,
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn owner_id(&self) -> u64 {
        self.owner_id
    }
}

impl Display for CreateProjectQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CreateProjectQueryView: title={} description={} owner_id={}",
            self.title,
            self.description.as_deref().unwrap_or(""),
            self.owner_id
        )
    }
}

impl DatabaseQueryView for CreateProjectQueryView {
    fn get_request(&self) -> String {
        "INSERT INTO projects (title, description, owner_id) VALUES ($1, $2, $3) RETURNING id"
            .to_string()
    }
}
