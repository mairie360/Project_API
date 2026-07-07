use std::fmt::Display;

use mairie360_api_lib::database::db_interface::DatabaseQueryView;

pub struct GetProjectsQueryView {
    user_id: u64,
}

impl GetProjectsQueryView {
    pub fn new(user_id: u64) -> Self {
        Self { user_id }
    }

    pub fn user_id(&self) -> u64 {
        self.user_id
    }
}

impl Display for GetProjectsQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GetProjectsQueryView: user_id={}", self.user_id)
    }
}

impl DatabaseQueryView for GetProjectsQueryView {
    fn get_request(&self) -> String {
        "SELECT DISTINCT p.id, p.title, p.description
        FROM projects p
        LEFT JOIN project_members pm ON p.id = pm.project_id
        WHERE p.owner_id = $1 OR pm.user_id = $1"
            .to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct ProjectView {
    id: i32,
    title: String,
    description: Option<String>,
}

impl ProjectView {
    pub fn new(id: i32, title: String, description: Option<&str>) -> Self {
        Self {
            id,
            title,
            description: description.map(|d| d.to_string()),
        }
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}
