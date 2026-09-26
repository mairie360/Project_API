use crate::endpoints::validation::{check_description, Validate, ValidationError};
use utoipa::ToSchema;

pub const MAX_COMMENT_LENGTH: usize = 2_000;

/// Commentaire à publier sur une tâche.
#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct AddTaskCommentView {
    /// Texte du commentaire, de 1 à 2000 caractères (espaces de bord ignorés).
    #[schema(
        min_length = 1,
        max_length = 2000,
        example = "La réunion publique est calée au 3 octobre."
    )]
    pub message: String,
}

impl Validate for AddTaskCommentView {
    fn validate(&self) -> Result<(), ValidationError> {
        check_description("message", &self.message, MAX_COMMENT_LENGTH)
    }
}
