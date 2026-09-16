use actix_web::web;
use utoipa::ToSchema;

use crate::endpoints::v1::projects::post::endpoint::CreateProjectError;

/// Projet à créer ; l'appelant en devient responsable.
#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct CreateProjectView {
    /// Nom du projet. Obligatoire.
    #[schema(example = "Réfection de la place du marché")]
    name: String,
    /// Description du projet. Facultative.
    #[schema(example = "Travaux de voirie 2026")]
    description: Option<String>,
    /// Groupe Core API dont tous les membres obtiennent l'accès au projet. Facultatif.
    #[schema(example = 3)]
    group_id: Option<u64>,
    // template_id: Option<u64>, ToDo
}

impl CreateProjectView {
    pub fn new(name: &str, description: Option<&str>, group_id: Option<u64>) -> Self {
        Self {
            name: name.to_string(),
            description: description.map(|d| d.to_string()),
            group_id,
        }
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

impl TryFrom<web::Json<CreateProjectView>> for CreateProjectView {
    type Error = CreateProjectError;

    fn try_from(params: web::Json<CreateProjectView>) -> Result<CreateProjectView, Self::Error> {
        Ok(params.into_inner())
    }
}

/// Projet créé.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct CreateProjectResultView {
    /// Identifiant attribué au projet créé.
    #[schema(example = 12)]
    pub project_id: u64,
}
