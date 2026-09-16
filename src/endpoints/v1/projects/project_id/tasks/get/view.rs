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
    /// Toujours une chaîne vide : la table des tâches ne stocke pas de description.
    #[schema(example = "")]
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
            description: "".to_string(),
            status: task.status().to_string().into(),
            priority: task.priority().to_string().into(),
            due_date: task.due_date(),
            assigned_to: task.assigned_to().map(|id| id as u64),
            fields: task.fields(),
        }
    }
}

/// Tâches d'un projet.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetTasksResultView {
    /// Tâches du projet. Vide si le projet n'en a aucune.
    pub tasks: Vec<TaskView>,
}
