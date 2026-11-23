use utoipa::ToSchema;

use crate::endpoints::validation::{check_description, Validate, ValidationError};

/// Longest project comment, once trimmed (Database `project_comments.message`).
pub const MAX_PROJECT_COMMENT_LENGTH: usize = 2_000;

/// Comment to post on a project (MAIR-1089).
#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct AddProjectCommentView {
    /// Text of the comment: 1 to 2000 characters once the leading and trailing spaces are removed,
    /// line breaks and tabs allowed, no other control character. `<` and `>` are ordinary text.
    #[schema(
        min_length = 1,
        max_length = 2000,
        example = "The tender closes on Friday."
    )]
    pub message: String,
}

impl Validate for AddProjectCommentView {
    fn validate(&self) -> Result<(), ValidationError> {
        let message = self.message.trim();
        if message.is_empty() {
            return Err(ValidationError::invalid("message", "must not be empty"));
        }
        check_description("message", message, MAX_PROJECT_COMMENT_LENGTH)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view(message: &str) -> AddProjectCommentView {
        AddProjectCommentView {
            message: message.to_string(),
        }
    }

    #[test]
    fn mair_1089_a_comment_is_not_blank_nor_too_long() {
        assert!(view("Budget > 10 000 € <3").validate().is_ok());
        assert!(view("  Line 1\nLine 2  ").validate().is_ok());
        assert!(view(&format!("  {}  ", "é".repeat(2000)))
            .validate()
            .is_ok());
        for refused in ["", "   ", "\n\t "] {
            assert_eq!(
                view(refused).validate().unwrap_err().to_string(),
                "Invalid `message`: must not be empty"
            );
        }
        assert!(view(&"a".repeat(2001)).validate().is_err());
        assert!(view("a\0b").validate().is_err());
    }
}
