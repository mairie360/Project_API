use mairie360_api_lib::database::db_interface::{
    id_from_sql, id_to_sql, ApiRequestDto, QueryParam,
};

use crate::database::project::update_status::view::ProjectStatus;
use crate::database::tasks::create_task::view::TaskPriority;

/// Creates a project and returns its id. Its status and priority default to the table's
/// (`active`, `medium`) unless `with_status` / `with_priority` set them (MAIR-558).
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
                QueryParam::Text(String::new()),
                QueryParam::Text(String::new()),
            ],
        }
    }

    /// Creates the project with this status instead of `active`.
    #[must_use]
    pub fn with_status(mut self, status: ProjectStatus) -> Self {
        self.params[3] = QueryParam::Text(status.to_string());
        self
    }

    /// Creates the project with this priority instead of `medium`.
    #[must_use]
    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.params[4] = QueryParam::Text(priority.to_string());
        self
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
        "INSERT INTO projects (title, description, owner_id, status, priority) \
         VALUES ($1, NULLIF($2, ''), $3, \
                 COALESCE(NULLIF($4, '')::project_status, 'active'), \
                 COALESCE(NULLIF($5, '')::task_priority, 'medium')) \
         RETURNING id"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }

    fn invalidated_read_caches(&self) -> &'static [&'static str] {
        crate::database::project::PROJECTS_READ_CACHES
    }
}
