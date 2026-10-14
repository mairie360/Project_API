use mairie360_api_lib::database::db_interface::{
    id_from_sql, id_to_sql, ApiRequestDto, QueryParam,
};

use crate::database::project::update_status::view::ProjectStatus;
use crate::database::tasks::create_task::view::TaskPriority;

/// Updates the title, the description, the status and/or the priority of a project (a missing
/// field is kept). Returns `true` if the project exists.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateProjectQueryView {
    params: Vec<QueryParam>,
}

impl UpdateProjectQueryView {
    pub fn new(
        project_id: u64,
        title: Option<&str>,
        description: Option<&str>,
        status: Option<ProjectStatus>,
    ) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(project_id)),
                QueryParam::Text(title.unwrap_or_default().to_string()),
                QueryParam::Bool(description.is_some()),
                QueryParam::Text(description.unwrap_or_default().to_string()),
                QueryParam::Text(status.map(|s| s.to_string()).unwrap_or_default()),
                QueryParam::Text(String::new()),
            ],
        }
    }

    /// Also sets the priority (MAIR-558).
    #[must_use]
    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.params[5] = QueryParam::Text(priority.to_string());
        self
    }

    pub fn project_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }
}

impl ApiRequestDto for UpdateProjectQueryView {
    fn query_sql(&self) -> &'static str {
        "WITH updated AS ( \
            UPDATE projects SET \
                title = COALESCE(NULLIF($2, ''), title), \
                description = CASE WHEN $3 THEN NULLIF($4, '') ELSE description END, \
                status = COALESCE(NULLIF($5, '')::project_status, status), \
                priority = COALESCE(NULLIF($6, '')::task_priority, priority) \
            WHERE id = $1 RETURNING id \
         ) \
         SELECT EXISTS (SELECT 1 FROM updated)"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }

    fn invalidated_read_caches(&self) -> &'static [&'static str] {
        crate::database::project::PROJECTS_READ_CACHES
    }
}
