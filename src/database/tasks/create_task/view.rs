use std::fmt::Display;

use mairie360_api_lib::database::db_interface::DatabaseQueryView;

#[derive(Debug, sqlx::Type, PartialEq, Eq, Clone, Copy)]
#[sqlx(type_name = "task_status", rename_all = "lowercase")] // Renomme tout en minuscule pour SQL
pub enum TaskStatus {
    Todo,
    #[sqlx(rename = "in_progress")]
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

impl ToString for TaskStatus {
    fn to_string(&self) -> String {
        match self {
            TaskStatus::Todo => "todo".to_string(),
            TaskStatus::InProgress => "in_progress".to_string(),
            TaskStatus::Completed => "completed".to_string(),
            TaskStatus::Error => "error".to_string(),
        }
    }
}

#[derive(Debug, sqlx::Type, PartialEq, Eq, Clone, Copy)]
#[sqlx(type_name = "task_priority", rename_all = "lowercase")]
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

impl ToString for TaskPriority {
    fn to_string(&self) -> String {
        match self {
            TaskPriority::Low => "low".to_string(),
            TaskPriority::Medium => "medium".to_string(),
            TaskPriority::High => "high".to_string(),
            TaskPriority::Error => "error".to_string(),
        }
    }
}

pub struct CreateTaskQueryView {
    project_id: u64,
    title: String,
    status: TaskStatus,
    priority: TaskPriority,
    due_date: Option<chrono::DateTime<chrono::Utc>>,
    assigned_to: Option<i32>,
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
            project_id,
            title: title.to_string(),
            status,
            priority,
            due_date,
            assigned_to: assigned_to.map(|id| id as i32),
        }
    }

    pub fn project_id(&self) -> u64 {
        self.project_id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn status(&self) -> TaskStatus {
        self.status
    }

    pub fn priority(&self) -> TaskPriority {
        self.priority
    }

    pub fn due_date(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.due_date
    }

    pub fn assigned_to(&self) -> Option<i32> {
        self.assigned_to
    }
}

impl Display for CreateTaskQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CreateTaskQueryView: project_id={} title={} status={} priority={} due_date={:?} assigned_to={:?}",
            self.project_id,
            self.title,
            self.status.to_string(),
            self.priority.to_string(),
            self.due_date,
            self.assigned_to
        )
    }
}

impl DatabaseQueryView for CreateTaskQueryView {
    fn get_request(&self) -> String {
        "INSERT INTO tasks (project_id, title, status, priority, due_date, assigned_to) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id"
            .to_string()
    }
}
