use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Completed,
    Error,
}

impl From<String> for TaskStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "todo" => TaskStatus::Todo,
            "in_progress" => TaskStatus::InProgress,
            "completed" => TaskStatus::Completed,
            _ => TaskStatus::Error,
        }
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TaskStatus::Todo => "todo",
            TaskStatus::InProgress => "in_progress",
            TaskStatus::Completed => "completed",
            TaskStatus::Error => "error",
        };
        f.write_str(s)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Error,
}

impl From<String> for TaskPriority {
    fn from(s: String) -> Self {
        match s.as_str() {
            "low" => TaskPriority::Low,
            "medium" => TaskPriority::Medium,
            "high" => TaskPriority::High,
            _ => TaskPriority::Error,
        }
    }
}

impl std::fmt::Display for TaskPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TaskPriority::Low => "low",
            TaskPriority::Medium => "medium",
            TaskPriority::High => "high",
            TaskPriority::Error => "error",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateTaskQueryView {
    params: Vec<QueryParam>,
}

impl CreateTaskQueryView {
    pub fn new(
        project_id: u64,
        title: &str,
        status: TaskStatus,
        priority: TaskPriority,
        due_date: Option<chrono::DateTime<chrono::Utc>>,
        assigned_to: Option<u64>,
    ) -> Self {
        Self {
            params: vec![
                QueryParam::I32(project_id as i32),
                QueryParam::Text(title.to_string()),
                QueryParam::Text(status.to_string()),
                QueryParam::Text(priority.to_string()),
                QueryParam::Text(due_date.map(|d| d.to_rfc3339()).unwrap_or_default()),
                QueryParam::OptionI32(assigned_to.map(|id| id as i32)),
            ],
        }
    }

    pub fn project_id(&self) -> u64 {
        self.params[0].as_i32() as u64
    }

    pub fn title(&self) -> &str {
        self.params[1].as_text()
    }
}

impl ApiRequestDto for CreateTaskQueryView {
    fn query_sql(&self) -> &'static str {
        "INSERT INTO tasks (project_id, title, status, priority, due_date, assigned_to) \
         VALUES ($1, $2, $3::task_status, $4::task_priority, NULLIF($5, '')::timestamptz, $6) \
         RETURNING id"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
