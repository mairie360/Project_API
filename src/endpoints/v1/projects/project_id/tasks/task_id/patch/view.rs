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

#[derive(Debug, Deserialize, ToSchema)]
pub struct PatchTaskView {
    pub name: Option<String>,
    /// Non persistée : la table `tasks` n'a pas de description.
    pub description: Option<String>,
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
    #[schema(value_type = Option<String>, format = DateTime)]
    pub due_date: Option<DateTime<Utc>>,
    /// Absent : assignation conservée ; `null` : assignation retirée.
    #[serde(default, deserialize_with = "deserialize_present")]
    #[schema(value_type = Option<u64>, nullable)]
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
