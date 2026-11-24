use mairie360_api_lib::database::db_interface::{
    id_from_sql, id_to_sql, ApiRequestDto, QueryParam,
};

use crate::database::project::update_status::view::ProjectStatus;
use crate::database::tasks::create_task::view::TaskPriority;

/// Creates a project, links its group and records its responsible (as a `manager` member) when they
/// are given, and returns its id. Its status and priority default to the table's
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
                QueryParam::OptionI32(None),
                QueryParam::OptionI32(None),
                QueryParam::OptionI32(None),
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

    /// Creates the project from this template (MAIR-566): the database copies its statuses and
    /// custom fields. Without it, the default template. An unknown id is a foreign key violation.
    #[must_use]
    pub fn with_template(mut self, template_id: Option<u64>) -> Self {
        self.params[5] = QueryParam::OptionI32(template_id.map(id_to_sql));
        self
    }

    /// Links this group to the project in the same statement. The caller checks first that the
    /// user may link it (`group::view::LinkableGroupQueryView`).
    #[must_use]
    pub fn with_group(mut self, group_id: Option<u64>) -> Self {
        self.params[6] = QueryParam::OptionI32(group_id.map(id_to_sql));
        self
    }

    /// Records this user as the responsible of the project and makes them a `manager` member in
    /// the same statement. An unknown user is a foreign key violation.
    #[must_use]
    pub fn with_responsible(mut self, responsible_id: Option<u64>) -> Self {
        self.params[7] = QueryParam::OptionI32(responsible_id.map(id_to_sql));
        self
    }

    /// Records the deadline entered for the project (`projects.due_date`), none without it.
    #[must_use]
    pub fn with_deadline(mut self, deadline: Option<chrono::NaiveDate>) -> Self {
        self.params[8] = QueryParam::Text(deadline.map(|day| day.to_string()).unwrap_or_default());
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
        "WITH created AS ( \
            INSERT INTO projects (title, description, owner_id, status, priority, template_id, \
                                  responsible_id, due_date) \
            VALUES ($1, NULLIF($2, ''), $3, \
                    COALESCE(NULLIF($4, '')::project_status, 'active'), \
                    COALESCE(NULLIF($5, '')::task_priority, 'medium'), $6, $8, \
                    NULLIF($9, '')::date) \
            RETURNING id \
         ), linked AS ( \
            INSERT INTO project_groups (project_id, group_id) \
            SELECT id, $7 FROM created WHERE $7::INT IS NOT NULL \
         ), responsible AS ( \
            INSERT INTO project_members (project_id, user_id, role) \
            SELECT id, $8, 'manager' FROM created WHERE $8::INT IS NOT NULL \
         ) \
         SELECT id FROM created"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }

    fn invalidated_read_caches(&self) -> &'static [&'static str] {
        crate::database::project::PROJECTS_READ_CACHES
    }
}
