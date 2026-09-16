use utoipa::ToSchema;

use crate::endpoints::v1::projects::get::view::ProjectStatus;

/// Modification partielle d'un projet : seuls les champs fournis sont mis à jour.
#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct UpdateProjectView {
    /// Nouveau nom. Absent ou `null` pour ne pas y toucher ; ne peut pas être réduit à des espaces.
    #[schema(min_length = 1, example = "Réfection de la place du marché")]
    pub name: Option<String>,
    /// Nouvelle description. Absent ou `null` pour ne pas y toucher.
    #[schema(example = "Travaux de voirie 2026, phase 2")]
    pub description: Option<String>,
    /// Nouveau statut. `Error` est refusé : ce n'est qu'une valeur de repli à la lecture.
    pub status: Option<ProjectStatus>,
}
