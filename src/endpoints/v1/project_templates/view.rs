use mairie360_api_lib::database::db_interface::id_from_sql;
use utoipa::ToSchema;

use crate::database::statuses::view::StatusCategory;
use crate::database::tasks::get_project_tasks::view::DynamicTaskField;
use crate::database::templates::view::{TemplateRow, TemplateStatusInput};
use crate::endpoints::validation::{
    check_color, check_description, check_label, check_optional, check_status_list, Validate,
    ValidationError, MAX_DESCRIPTION_LENGTH, MAX_STATUS_NAME_LENGTH, MAX_TITLE_LENGTH,
};

/// A task status of a template, copied into the projects created from it.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct TemplateStatusView {
    #[schema(example = "À valider")]
    pub name: String,
    /// `#RRGGBB`.
    #[schema(example = "#D97706")]
    pub color: String,
    pub category: StatusCategory,
    /// Position in the template, from 0.
    #[schema(example = 2)]
    pub position: u32,
}

/// A project template (MAIR-566).
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct ProjectTemplateView {
    #[schema(example = 2)]
    pub id: u64,
    #[schema(example = "Avec validation")]
    pub name: String,
    #[schema(example = "Les tâches terminées attendent une validation.")]
    pub description: Option<String>,
    /// The template of a project created without `template_id`; it cannot be deleted.
    pub is_default: bool,
    /// Task statuses, by position.
    pub statuses: Vec<TemplateStatusView>,
    /// Custom fields copied into the projects created from it.
    pub custom_fields: Vec<DynamicTaskField>,
}

impl From<TemplateRow> for ProjectTemplateView {
    fn from(row: TemplateRow) -> Self {
        Self {
            id: id_from_sql(row.id),
            name: row.name,
            description: row.description,
            is_default: row.is_default,
            statuses: row
                .statuses
                .into_iter()
                .filter_map(|status| {
                    Some(TemplateStatusView {
                        category: StatusCategory::from_stored(&status.category)?,
                        name: status.name,
                        color: status.color,
                        position: u32::try_from(status.position).unwrap_or_default(),
                    })
                })
                .collect(),
            custom_fields: row
                .custom_field_schema
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|field| serde_json::from_value(field.clone()).ok())
                .collect(),
        }
    }
}

/// Every project template, the default one first, then by name.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetTemplatesResultView {
    pub templates: Vec<ProjectTemplateView>,
}

/// A status of a template to write; its position is its index in the list.
#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct TemplateStatusInputView {
    /// 1 to 100 characters, not blank, no control character; unique in the template, whatever the
    /// case.
    #[schema(min_length = 1, max_length = 100, example = "À valider")]
    pub name: String,
    /// `#RRGGBB`. Absent: `#64748B`.
    #[schema(example = "#D97706")]
    pub color: Option<String>,
    pub category: StatusCategory,
}

impl TemplateStatusInputView {
    pub fn as_input(&self) -> TemplateStatusInput<'_> {
        TemplateStatusInput {
            name: self.name.trim(),
            color: self.color.as_deref(),
            category: self.category,
        }
    }
}

fn check_statuses(statuses: &[TemplateStatusInputView]) -> Result<(), ValidationError> {
    for status in statuses {
        check_label("statuses.name", &status.name, MAX_STATUS_NAME_LENGTH)?;
        check_optional(status.color.as_deref(), |color| {
            check_color("statuses.color", color)
        })?;
    }
    let list: Vec<(&str, StatusCategory)> = statuses
        .iter()
        .map(|status| (status.name.as_str(), status.category))
        .collect();
    check_status_list("statuses", &list, &StatusCategory::ALL)
}

fn check_fields(fields: &[DynamicTaskField]) -> Result<(), ValidationError> {
    for field in fields {
        check_label("custom_fields.label", &field.label, MAX_TITLE_LENGTH)?;
    }
    Ok(())
}

/// A template to create.
#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct CreateTemplateView {
    /// 1 to 100 characters, not blank, no control character; unique whatever the case.
    #[schema(min_length = 1, max_length = 100, example = "Travaux de voirie")]
    pub name: String,
    /// At most 5000 characters. Optional.
    #[schema(max_length = 5000, example = "Étude, chantier, réception.")]
    pub description: Option<String>,
    /// 1 to 30 statuses, in order, at least one of each category.
    pub statuses: Vec<TemplateStatusInputView>,
    /// Custom fields of the projects created from it. Absent: none.
    #[serde(default)]
    pub custom_fields: Vec<DynamicTaskField>,
}

impl Validate for CreateTemplateView {
    fn validate(&self) -> Result<(), ValidationError> {
        check_label("name", &self.name, MAX_STATUS_NAME_LENGTH)?;
        check_optional(self.description.as_deref(), |description| {
            check_description("description", description, MAX_DESCRIPTION_LENGTH)
        })?;
        check_statuses(&self.statuses)?;
        check_fields(&self.custom_fields)
    }
}

/// Template created.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct CreateTemplateResultView {
    #[schema(example = 5)]
    pub template_id: u64,
}

/// Partial update of a template: an absent field is left unchanged. The projects already created
/// from it keep their statuses.
#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct PatchTemplateView {
    #[schema(min_length = 1, max_length = 100, example = "Travaux de voirie")]
    pub name: Option<String>,
    /// `""` clears it.
    #[schema(max_length = 5000)]
    pub description: Option<String>,
    /// The whole new list of statuses, same rules as on creation.
    pub statuses: Option<Vec<TemplateStatusInputView>>,
    /// The whole new list of custom fields.
    pub custom_fields: Option<Vec<DynamicTaskField>>,
}

impl Validate for PatchTemplateView {
    fn validate(&self) -> Result<(), ValidationError> {
        check_optional(self.name.as_deref(), |name| {
            check_label("name", name, MAX_STATUS_NAME_LENGTH)
        })?;
        check_optional(self.description.as_deref(), |description| {
            check_description("description", description, MAX_DESCRIPTION_LENGTH)
        })?;
        if let Some(statuses) = &self.statuses {
            check_statuses(statuses)?;
        }
        if let Some(fields) = &self.custom_fields {
            check_fields(fields)?;
        }
        Ok(())
    }
}
