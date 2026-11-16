use crate::endpoints::validation::{check_description, Validate, ValidationError};
use utoipa::ToSchema;

pub const MAX_COMMENT_LENGTH: usize = 2_000;

/// Comment to post on a task.
#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct AddTaskCommentView {
    /// Text of the comment, 1 to 2000 characters (leading and trailing spaces ignored).
    #[schema(
        min_length = 1,
        max_length = 2000,
        example = "The public meeting is set for 3 October."
    )]
    pub message: String,
}

impl Validate for AddTaskCommentView {
    fn validate(&self) -> Result<(), ValidationError> {
        check_description("message", &self.message, MAX_COMMENT_LENGTH)
    }
}
