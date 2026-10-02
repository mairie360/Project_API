use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// One page of the projects visible to the user (see `project_visible_to_user_sql!`), newest first.
/// Read with `fetch_one::<PagedRows<ProjectView>, _>`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetProjectsQueryView {
    params: Vec<QueryParam>,
}

impl GetProjectsQueryView {
    pub fn new(user_id: u64, limit: u32, offset: u32) -> Self {
        Self {
            params: vec![
                QueryParam::I32(user_id as i32),
                QueryParam::I64(i64::from(limit)),
                QueryParam::I64(i64::from(offset)),
            ],
        }
    }

    pub fn user_id(&self) -> u64 {
        self.params[0].as_i32() as u64
    }
}

impl ApiRequestDto for GetProjectsQueryView {
    fn query_sql(&self) -> &'static str {
        concat!(
            crate::paged_rows_sql!("$2", "$3"),
            " FROM ( \
                SELECT p.id, p.title, p.description, p.status, \
                       row_number() OVER (ORDER BY p.created_at DESC NULLS LAST, p.id DESC) AS rn \
                FROM projects p \
                WHERE ",
            crate::project_visible_to_user_sql!(),
            " ) t"
        )
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProjectView {
    id: i32,
    title: String,
    description: Option<String>,
    status: String,
}

impl ProjectView {
    pub fn new(id: i32, title: &str, description: Option<&str>, status: &str) -> Self {
        Self {
            id,
            title: title.to_string(),
            description: description.map(str::to_string),
            status: status.to_string(),
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

    pub fn status(&self) -> &str {
        &self.status
    }
}
