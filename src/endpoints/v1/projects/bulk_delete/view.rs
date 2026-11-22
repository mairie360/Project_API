use utoipa::ToSchema;

use crate::endpoints::validation::{Validate, ValidationError};

/// Most projects one bulk deletion takes.
pub const MAX_BULK_DELETE: usize = 100;

/// Projects to delete (MAIR-1089).
#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct BulkDeleteProjectsView {
    /// Ids of the projects, 1 to 100. A repeated id is handled once.
    #[schema(min_items = 1, max_items = 100, example = json!([12, 18, 25]))]
    pub ids: Vec<u64>,
}

impl BulkDeleteProjectsView {
    /// The ids in the order given, each once.
    pub fn unique_ids(&self) -> Vec<u64> {
        let mut seen = std::collections::HashSet::new();
        self.ids
            .iter()
            .copied()
            .filter(|id| seen.insert(*id))
            .collect()
    }
}

impl Validate for BulkDeleteProjectsView {
    fn validate(&self) -> Result<(), ValidationError> {
        if self.ids.is_empty() || self.ids.len() > MAX_BULK_DELETE {
            return Err(ValidationError::invalid(
                "ids",
                &format!("must hold 1 to {MAX_BULK_DELETE} ids"),
            ));
        }
        Ok(())
    }
}

/// What happened to one project of a bulk deletion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum BulkDeleteStatus {
    /// The project was deleted, with its tasks, milestones, comments and memberships.
    Deleted,
    /// No such project, or one the caller cannot see.
    NotFound,
    /// The caller sees the project but may not delete it (not one of its managers).
    Forbidden,
}

/// Outcome for one id.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, ToSchema)]
pub struct BulkDeleteResultItem {
    #[schema(example = 12)]
    pub id: u64,
    pub status: BulkDeleteStatus,
}

/// Outcome of a bulk deletion, one item per distinct id in the order given.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct BulkDeleteProjectsResultView {
    pub results: Vec<BulkDeleteResultItem>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(body: serde_json::Value) -> Result<BulkDeleteProjectsView, serde_json::Error> {
        serde_json::from_value(body)
    }

    #[test]
    fn mair_1089_one_to_a_hundred_ids_each_once() {
        let view = parse(serde_json::json!({ "ids": [12, 18, 12, 25] })).unwrap();
        assert!(view.validate().is_ok());
        assert_eq!(view.unique_ids(), vec![12, 18, 25]);
        let hundred: Vec<u64> = (1..=100).collect();
        assert!(parse(serde_json::json!({ "ids": hundred }))
            .unwrap()
            .validate()
            .is_ok());
        let too_many: Vec<u64> = (1..=101).collect();
        for refused in [
            serde_json::json!({ "ids": [] }),
            serde_json::json!({ "ids": too_many }),
        ] {
            assert_eq!(
                parse(refused).unwrap().validate().unwrap_err().to_string(),
                "Invalid `ids`: must hold 1 to 100 ids"
            );
        }
        for invalid in [
            serde_json::json!({}),
            serde_json::json!({ "ids": [-1] }),
            serde_json::json!({ "ids": ["12"] }),
        ] {
            assert!(parse(invalid).is_err());
        }
        assert_eq!(
            serde_json::to_value(BulkDeleteStatus::NotFound).unwrap(),
            serde_json::json!("not_found")
        );
    }
}
