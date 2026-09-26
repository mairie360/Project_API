use crate::endpoints::validation::{
    check_description, check_json, check_label, check_optional, Validate, ValidationError,
    MAX_DESCRIPTION_LENGTH, MAX_TITLE_LENGTH,
};
use crate::{
    database::tasks::get_project_tasks::view::DynamicTaskField,
    endpoints::v1::projects::project_id::get::view::{TaskPriority, TaskStatus},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer};
use utoipa::ToSchema;

/// Distingue un champ absent (`None`) d'un champ explicitement `null` (`Some(None)`).
fn deserialize_present<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

/// Modification partielle d'une tâche : seuls les champs fournis sont mis à jour.
#[derive(Debug, Deserialize, ToSchema)]
pub struct PatchTaskView {
    /// Nouvel intitulé. Absent pour ne pas y toucher. Interdit à l'agent assigné non gestionnaire.
    #[schema(
        min_length = 1,
        max_length = 255,
        example = "Consulter les riverains et les commerçants"
    )]
    pub name: Option<String>,
    /// Non persistée : la table `tasks` n'a pas de description.
    #[schema(max_length = 5000, example = "Réunion publique avec les riverains")]
    pub description: Option<String>,
    /// Nouveau statut. Seul champ qu'un agent assigné non gestionnaire a le droit de modifier.
    pub status: Option<TaskStatus>,
    /// Nouvelle priorité. Interdite à l'agent assigné non gestionnaire.
    pub priority: Option<TaskPriority>,
    /// Nouvelle échéance. Interdite à l'agent assigné non gestionnaire.
    #[schema(value_type = Option<String>, format = DateTime, example = "2026-10-15T00:00:00Z")]
    pub due_date: Option<DateTime<Utc>>,
    /// Absent : assignation conservée ; `null` : assignation retirée.
    #[serde(default, deserialize_with = "deserialize_present")]
    #[schema(value_type = Option<u64>, nullable, example = 42)]
    pub assigned_to: Option<Option<u64>>,
    /// Non persistés par cette opération.
    pub fields: Option<Vec<DynamicTaskField>>,
}

impl PatchTaskView {
    /// Vrai si le corps ne modifie que le statut (seule modification permise à l'agent assigné).
    pub fn only_changes_status(&self) -> bool {
        self.status.is_some()
            && self.name.is_none()
            && self.description.is_none()
            && self.priority.is_none()
            && self.due_date.is_none()
            && self.assigned_to.is_none()
            && self.fields.is_none()
    }
}

impl Validate for PatchTaskView {
    fn validate(&self) -> Result<(), ValidationError> {
        check_optional(self.name.as_deref(), |name| {
            check_label("name", name.trim(), MAX_TITLE_LENGTH)
        })?;
        check_optional(self.description.as_deref(), |description| {
            check_description("description", description, MAX_DESCRIPTION_LENGTH)
        })?;
        for field in self.fields.iter().flatten() {
            check_label("fields.label", &field.label, MAX_TITLE_LENGTH)?;
            for option in &field.fields_options {
                check_json("fields.fields_options.option", &option.option)?;
            }
        }
        Ok(())
    }
}
