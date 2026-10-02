use chrono::{DateTime, Utc};
use utoipa::ToSchema;

use crate::{
    database::tasks::get_project_tasks::view::{DynamicTaskField, Task},
    endpoints::v1::projects::project_id::get::view::{TaskPriority, TaskStatus},
};

/// Tâche d'un projet.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct TaskView {
    /// Identifiant de la tâche.
    #[schema(example = 77)]
    pub id: u64,
    /// Intitulé de la tâche.
    #[schema(example = "Consulter les riverains")]
    pub title: String,
    /// Description of the task; empty string if it has none — never `null`.
    #[schema(example = "Réunion publique à organiser avant le 15 octobre")]
    pub description: String,
    /// Statut courant. `Error` signale une valeur en base que l'API ne sait pas interpréter.
    pub status: TaskStatus,
    /// Priorité. `Error` signale une valeur en base que l'API ne sait pas interpréter.
    pub priority: TaskPriority,
    /// Échéance, ou `null` si la tâche n'en a pas.
    #[schema(value_type = Option<String>, format = DateTime, example = "2026-10-15T00:00:00Z")]
    pub due_date: Option<DateTime<Utc>>,
    /// Identifiant Core API de l'agent assigné, ou `null` si la tâche n'est assignée à personne.
    #[schema(example = 42)]
    pub assigned_to: Option<u64>,
    /// Champs personnalisés définis sur le projet. Vide s'il n'y en a aucun.
    pub fields: Vec<DynamicTaskField>,
}

impl From<Task> for TaskView {
    fn from(task: Task) -> Self {
        TaskView {
            id: task.id() as u64,
            title: task.title().to_string(),
            description: task.description().to_string(),
            status: task.status().to_string().into(),
            priority: task.priority().to_string().into(),
            due_date: task.due_date(),
            assigned_to: task.assigned_to().map(|id| id as u64),
            fields: task.fields(),
        }
    }
}

/// One page of the tasks of a project.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetTasksResultView {
    /// Tasks of the page, oldest first. Empty if the project has none or the page is past the end.
    pub tasks: Vec<TaskView>,
    /// Number of tasks of the project, whatever the page.
    #[schema(example = 1)]
    pub total: u64,
}
