use utoipa::ToSchema;

use crate::endpoints::v1::projects::get::view::ProjetView;
use crate::endpoints::v1::projects::project_id::tasks::get::view::TaskView;
use crate::endpoints::v1::projects::project_id::users::get::view::User;

/// Statut d'une tâche : `Todo` (à faire), `InProgress` (en cours) ou `Completed` (terminée).
/// `Error` n'est jamais assignable : il signale à la lecture une valeur en base inconnue.
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize, ToSchema)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Completed,
    Error,
}

impl From<String> for TaskStatus {
    fn from(value: String) -> Self {
        match value.as_str() {
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

/// Priorité d'une tâche : `Low`, `Medium`, `High` ou `Urgent`.
/// `Error` n'est jamais assignable : il signale à la lecture une valeur en base inconnue.
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize, ToSchema)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Urgent,
    Error,
}

impl From<String> for TaskPriority {
    fn from(value: String) -> Self {
        match value.as_str() {
            "low" => TaskPriority::Low,
            "medium" => TaskPriority::Medium,
            "high" => TaskPriority::High,
            "urgent" => TaskPriority::Urgent,
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
            TaskPriority::Urgent => "urgent",
            TaskPriority::Error => "error",
        };
        f.write_str(s)
    }
}

/// Project visible to the caller, with one page of its tasks and all its members.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetProjectResultView {
    /// The requested project.
    pub project: ProjetView,
    /// One page of its tasks, oldest first. Empty if the project has none or the page is past the end.
    pub tasks: Vec<TaskView>,
    /// Number of tasks of the project, whatever the page.
    #[schema(example = 1)]
    pub tasks_total: u64,
    /// All its members.
    pub users: Vec<User>,
}
