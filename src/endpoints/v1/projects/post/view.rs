use utoipa::ToSchema;

use crate::endpoints::v1::projects::get::view::ProjectStatus;
use crate::endpoints::v1::projects::project_id::get::view::TaskPriority;

use crate::endpoints::validation::{
    check_description, check_label, check_optional, Validate, ValidationError,
    MAX_DESCRIPTION_LENGTH, MAX_TITLE_LENGTH,
};

/// Projet à créer ; l'appelant en devient responsable.
#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct CreateProjectView {
    /// Nom du projet. Obligatoire.
    #[schema(
        min_length = 1,
        max_length = 255,
        example = "Réfection de la place du marché"
    )]
    name: String,
    /// Description du projet. Facultative.
    #[schema(max_length = 5000, example = "Travaux de voirie 2026")]
    description: Option<String>,
    /// Groupe Core API dont tous les membres obtiennent l'accès au projet. Facultatif.
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

/// Projet créé.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct CreateProjectResultView {
    /// Identifiant attribué au projet créé.
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
