use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ProjectStatus {
    Active,
    Suspended,
    Archived,
    Completed,
    Error,
}

impl From<String> for ProjectStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "suspended" => ProjectStatus::Suspended,
            "archived" => ProjectStatus::Archived,
            "completed" => ProjectStatus::Completed,
            "error" => ProjectStatus::Error,
            _ => ProjectStatus::Active,
        }
    }
}

impl std::fmt::Display for ProjectStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ProjectStatus::Active => "active",
            ProjectStatus::Suspended => "suspended",
            ProjectStatus::Archived => "archived",
            ProjectStatus::Completed => "completed",
            ProjectStatus::Error => "error",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateProjectStatusQueryView {
    params: Vec<QueryParam>,
}

impl UpdateProjectStatusQueryView {
    pub fn new(project_id: u64, status: ProjectStatus) -> Self {
        Self {
            params: vec![
                QueryParam::Text(status.to_string()),
                QueryParam::I32(project_id as i32),
            ],
        }
    }

    pub fn status(&self) -> ProjectStatus {
        self.params[0].as_text().to_string().into()
    }

    pub fn project_id(&self) -> u64 {
        self.params[1].as_i32() as u64
    }
}

impl ApiRequestDto for UpdateProjectStatusQueryView {
    fn query_sql(&self) -> &'static str {
        "UPDATE projects SET status = $1::project_status WHERE id = $2"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
