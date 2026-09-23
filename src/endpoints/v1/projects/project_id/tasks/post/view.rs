use chrono::{DateTime, Utc};
use utoipa::ToSchema;

use crate::endpoints::validation::{
    check_description, check_json, check_label, check_optional, Validate, ValidationError,
    MAX_DESCRIPTION_LENGTH, MAX_TITLE_LENGTH,
};
use crate::{
    database::tasks::get_project_tasks::view::DynamicTaskField,
    endpoints::v1::projects::project_id::get::view::{TaskPriority, TaskStatus},
};

/// Tâche à créer dans le projet du chemin.
#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct CreateTaskView {
    /// Intitulé de la tâche. Obligatoire.
    #[schema(min_length = 1, max_length = 255, example = "Consulter les riverains")]
    name: String,
    /// Description. Renvoyée dans la réponse mais **non persistée** : les lectures ultérieures
    /// la donneront vide.
    #[schema(
        max_length = 5000,
        example = "Réunion publique à organiser avant le 15 octobre"
    )]
    description: Option<String>,
    /// Échéance. Facultative.
    #[schema(value_type = Option<String>, format = DateTime, example = "2026-10-15T00:00:00Z")]
    due_date: Option<DateTime<Utc>>,
    /// Statut initial. Absent, la base applique sa valeur par défaut.
    status: Option<TaskStatus>,
    /// Priorité initiale. Absente, la base applique sa valeur par défaut.
    priority: Option<TaskPriority>,
    /// Identifiant Core API de l'agent à assigner. Facultatif.
    #[schema(example = 42)]
    assigned_to: Option<u64>,
    /// Champs personnalisés. Obligatoire, quitte à être un tableau vide.
    fields: Vec<DynamicTaskField>,
}

impl CreateTaskView {
    pub fn new(name: String, fields: Vec<DynamicTaskField>) -> Self {
        Self {
            name,
            description: None,
            due_date: None,
            status: None,
            priority: None,
            assigned_to: None,
            fields,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &Option<String> {
        &self.description
    }

    pub fn due_date(&self) -> &Option<DateTime<Utc>> {
        &self.due_date
    }

    pub fn status(&self) -> Option<TaskStatus> {
        self.status.clone()
    }

    pub fn priority(&self) -> Option<TaskPriority> {
        self.priority.clone()
    }

    pub fn assigned_to(&self) -> &Option<u64> {
        &self.assigned_to
    }

    pub fn fields(&self) -> &Vec<DynamicTaskField> {
        &self.fields
    }
}

/// Tâche créée.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct CreateTaskResultView {
    /// Identifiant attribué à la tâche créée.
    #[schema(example = 77)]
    pub task_id: u64,
    /// Intitulé enregistré.
    #[schema(example = "Consulter les riverains")]
    pub name: String,
    /// Description telle qu'envoyée. Elle n'est pas persistée et ne sera pas relue.
    #[schema(example = "Réunion publique à organiser avant le 15 octobre")]
    pub description: Option<String>,
}

impl Validate for CreateTaskView {
    fn validate(&self) -> Result<(), ValidationError> {
        check_label("name", &self.name, MAX_TITLE_LENGTH)?;
        check_optional(self.description.as_deref(), |description| {
            check_description("description", description, MAX_DESCRIPTION_LENGTH)
        })?;
        for field in &self.fields {
            check_label("fields.label", &field.label, MAX_TITLE_LENGTH)?;
            for option in &field.fields_options {
                check_json("fields.fields_options.option", &option.option)?;
            }
        }
        Ok(())
    }
}
