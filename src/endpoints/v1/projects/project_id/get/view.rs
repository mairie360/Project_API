use utoipa::ToSchema;

use crate::endpoints::v1::projects::get::view::ProjetView;
use crate::endpoints::v1::projects::project_id::tasks::get::view::TaskView;
use crate::endpoints::v1::projects::project_id::users::get::view::User;

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

/// Projet visible par l'appelant, avec ses tâches et ses membres.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetProjectResultView {
    /// Le projet demandé.
    pub project: ProjetView,
    /// Toutes ses tâches. Vide si le projet n'en a aucune.
    pub tasks: Vec<TaskView>,
    /// Tous ses membres.
    pub users: Vec<User>,
}
