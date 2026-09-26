use utoipa::ToSchema;

use crate::endpoints::v1::projects::get::view::ProjectStatus;
use crate::endpoints::validation::{
    check_description, check_label, check_optional, Validate, ValidationError,
    MAX_DESCRIPTION_LENGTH, MAX_TITLE_LENGTH,
};

/// Modification partielle d'un projet : seuls les champs fournis sont mis à jour.
#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct UpdateProjectView {
    /// Nouveau nom. Absent ou `null` pour ne pas y toucher ; ne peut pas être réduit à des espaces.
    #[schema(
        min_length = 1,
        max_length = 255,
        example = "Réfection de la place du marché"
    )]
    pub name: Option<String>,
    /// Nouvelle description. Absent ou `null` pour ne pas y toucher.
    #[schema(max_length = 5000, example = "Travaux de voirie 2026, phase 2")]
    pub description: Option<String>,
    /// Nouveau statut. `Error` est refusé : ce n'est qu'une valeur de repli à la lecture.
    pub status: Option<ProjectStatus>,
}

impl Validate for UpdateProjectView {
    fn validate(&self) -> Result<(), ValidationError> {
        check_optional(self.name.as_deref(), |name| {
            check_label("name", name.trim(), MAX_TITLE_LENGTH)
        })?;
        check_optional(self.description.as_deref(), |description| {
            check_description("description", description, MAX_DESCRIPTION_LENGTH)
        })
    }
}
