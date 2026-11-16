use utoipa::ToSchema;

use crate::endpoints::v1::projects::get::view::ProjectStatus;
use crate::endpoints::v1::projects::project_id::get::view::TaskPriority;

use crate::endpoints::validation::{
    check_description, check_label, check_optional, Validate, ValidationError,
    MAX_DESCRIPTION_LENGTH, MAX_TITLE_LENGTH,
};

/// Project to create; the caller becomes its manager.
#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct CreateProjectView {
    /// Name of the project. Required.
    #[schema(
        min_length = 1,
        max_length = 255,
        example = "Renovation of the market square"
    )]
    name: String,
    /// Description du projet. Facultative.
    #[schema(max_length = 5000, example = "2026 road works")]
    description: Option<String>,
    /// Core API group whose members all get access to the project. Optional.
    #[schema(example = 3)]
    group_id: Option<u64>,
    /// Initial status. Absent or `null` for `Active`; `Error` is refused (`400`).
    #[serde(default)]
    status: Option<ProjectStatus>,
    /// Priority of the project. Absent or `null` for `Medium`; `Urgent` is stored `High`, `Error`
    /// is refused (`400`).
    #[serde(default)]
    priority: Option<TaskPriority>,
    /// Template the project is created from (`GET /api/v1/project-templates/`, MAIR-566): its
    /// task statuses and custom fields are copied into the project. Absent or `null` for the
    /// default template.
    #[serde(default)]
    #[schema(example = 2)]
    template_id: Option<u64>,
}

impl CreateProjectView {
    pub fn new(name: &str, description: Option<&str>, group_id: Option<u64>) -> Self {
        Self {
            name: name.to_string(),
            description: description.map(|d| d.to_string()),
            group_id,
            status: None,
            priority: None,
            template_id: None,
        }
    }

    pub fn status(&self) -> Option<ProjectStatus> {
        self.status
    }

    pub fn priority(&self) -> Option<&TaskPriority> {
        self.priority.as_ref()
    }

    pub fn template_id(&self) -> Option<u64> {
        self.template_id
    }

    /// Creates the project from this template.
    #[must_use]
    pub fn with_template_id(mut self, template_id: u64) -> Self {
        self.template_id = Some(template_id);
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn group_id(&self) -> Option<u64> {
        self.group_id
    }
}

/// Project created.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct CreateProjectResultView {
    /// Id assigned to the created project.
    #[schema(example = 12)]
    pub project_id: u64,
}

impl Validate for CreateProjectView {
    fn validate(&self) -> Result<(), ValidationError> {
        check_label("name", &self.name, MAX_TITLE_LENGTH)?;
        check_optional(self.description.as_deref(), |description| {
            check_description("description", description, MAX_DESCRIPTION_LENGTH)
        })
    }
}
