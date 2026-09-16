use utoipa::ToSchema;

use crate::database::project::get_projects::view::ProjectView;

/// Statut d'un projet : `Active` (en cours), `Suspended` (suspendu) ou `Completed` (terminé).
/// `Error` n'est jamais assignable : il signale à la lecture une valeur en base inconnue.
#[derive(Debug, serde::Serialize, serde::Deserialize, ToSchema)]
pub enum ProjectStatus {
    /// Projet en cours.
    Active,
    Suspended,
    Completed,
    Error,
}

impl From<String> for ProjectStatus {
    fn from(value: String) -> Self {
        match value.as_str() {
            "active" => ProjectStatus::Active,
            "suspended" => ProjectStatus::Suspended,
            "completed" => ProjectStatus::Completed,
            _ => ProjectStatus::Error,
        }
    }
}

impl std::fmt::Display for ProjectStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ProjectStatus::Active => "active",
            ProjectStatus::Suspended => "suspended",
            ProjectStatus::Completed => "completed",
            ProjectStatus::Error => "error",
        };
        f.write_str(s)
    }
}

/// Projet, tel que listé ou consulté.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct ProjetView {
    /// Identifiant du projet, à réutiliser dans `/api/v1/projects/{project_id}/`.
    #[schema(example = 12)]
    pub id: u64,
    /// Nom du projet.
    #[schema(example = "Réfection de la place du marché")]
    pub name: String,
    /// Description du projet. Chaîne vide s'il n'en a pas — jamais `null`.
    #[schema(example = "Travaux de voirie 2026")]
    pub description: String,
    /// Statut courant. `Error` signale une valeur en base que l'API ne sait pas interpréter ;
    /// ce n'est pas un statut assignable.
    pub status: ProjectStatus,
}

impl From<ProjectView> for ProjetView {
    fn from(value: ProjectView) -> Self {
        Self {
            id: value.id() as u64,
            name: value.title().to_string(),
            description: value.description().unwrap_or_default().to_string(),
            status: value.status().to_string().into(),
        }
    }
}

/// Projets visibles par l'utilisateur connecté.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetProjectsResultView {
    /// Projets visibles par l'utilisateur connecté. Vide s'il n'a accès à aucun projet.
    pub projects: Vec<ProjetView>,
}
