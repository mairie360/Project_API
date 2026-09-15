use utoipa::ToSchema;

pub const MAX_COMMENT_LENGTH: usize = 2_000;

#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct AddTaskCommentView {
    /// Texte du commentaire, de 1 à 2000 caractères (espaces de bord ignorés).
    pub message: String,
}
