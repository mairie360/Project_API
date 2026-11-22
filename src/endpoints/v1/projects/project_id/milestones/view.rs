use chrono::{DateTime, NaiveDate, Utc};
use mairie360_api_lib::database::db_interface::id_from_sql;
use utoipa::ToSchema;

use crate::database::milestones::view::MilestoneRow;
use crate::endpoints::validation::{
    check_label, check_optional, Validate, ValidationError, MAX_TITLE_LENGTH,
};

/// A milestone of a project: a dated step it must reach (MAIR-1086).
#[derive(Debug, Clone, serde::Serialize, ToSchema)]
pub struct MilestoneView {
    /// Id of the milestone, to reuse in `…/milestones/{milestone_id}/`.
    #[schema(example = 5)]
    pub id: u64,
    /// Id of its project.
    #[schema(example = 12)]
    pub project_id: u64,
    /// Title of the milestone.
    #[schema(example = "Tender awarded")]
    pub title: String,
    /// Day the milestone is due, `YYYY-MM-DD`.
    #[schema(value_type = String, format = Date, example = "2026-12-15")]
    pub due_date: NaiveDate,
    /// `true` once the milestone is reached.
    #[schema(example = false)]
    pub reached: bool,
    /// Core API id of the user who created it, `null` when unknown.
    #[schema(example = 42)]
    pub created_by: Option<u64>,
    /// Core API id of the user who last changed it, `null` when unknown.
    #[schema(example = 42)]
    pub updated_by: Option<u64>,
    /// Creation date, RFC 3339.
    #[schema(value_type = String, format = DateTime, example = "2026-11-21T09:30:00Z")]
    pub created_at: DateTime<Utc>,
    /// Date of the last change, RFC 3339.
    #[schema(value_type = String, format = DateTime, example = "2026-11-21T09:30:00Z")]
    pub updated_at: DateTime<Utc>,
}

impl From<MilestoneRow> for MilestoneView {
    fn from(row: MilestoneRow) -> Self {
        Self {
            id: id_from_sql(row.id),
            project_id: id_from_sql(row.project_id),
            title: row.title,
            due_date: row.due_date,
            reached: row.reached,
            created_by: row.created_by.map(id_from_sql),
            updated_by: row.updated_by.map(id_from_sql),
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// One page of the milestones of a project.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetProjectMilestonesResultView {
    /// Milestones of the page, by due date then id. Empty if the project has none or the page is
    /// past the end.
    pub milestones: Vec<MilestoneView>,
    /// Number of milestones of the project (deleted ones excluded), whatever the page.
    #[schema(example = 3)]
    pub total: u64,
}

/// Milestone to add to the project of the path.
#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct CreateMilestoneView {
    /// Title of the milestone: 1 to 255 characters, not blank, no control character. Stored
    /// trimmed.
    #[schema(min_length = 1, max_length = 255, example = "Tender awarded")]
    pub title: String,
    /// Day the milestone is due, `YYYY-MM-DD`.
    #[schema(value_type = String, format = Date, example = "2026-12-15")]
    pub due_date: NaiveDate,
    /// `true` if the milestone is already reached. Absent: `false`.
    #[serde(default)]
    #[schema(example = false)]
    pub reached: bool,
}

impl Validate for CreateMilestoneView {
    fn validate(&self) -> Result<(), ValidationError> {
        check_label("title", self.title.trim(), MAX_TITLE_LENGTH)
    }
}

/// Partial update of a milestone: only the given fields change.
#[derive(Debug, Default, serde::Deserialize, ToSchema)]
pub struct PatchMilestoneView {
    /// New title (same rules as the creation). Absent or `null` to keep it.
    #[schema(
        min_length = 1,
        max_length = 255,
        example = "Tender awarded to the lowest bidder"
    )]
    pub title: Option<String>,
    /// New due day, `YYYY-MM-DD`. Absent or `null` to keep it.
    #[schema(value_type = Option<String>, format = Date, example = "2027-01-15")]
    pub due_date: Option<NaiveDate>,
    /// `true` once reached, `false` to reopen it. Absent or `null` to keep it.
    #[schema(example = true)]
    pub reached: Option<bool>,
}

impl Validate for PatchMilestoneView {
    fn validate(&self) -> Result<(), ValidationError> {
        check_optional(self.title.as_deref(), |title| {
            check_label("title", title.trim(), MAX_TITLE_LENGTH)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mair_1086_a_milestone_needs_a_title_and_a_day() {
        let parse = |body: serde_json::Value| serde_json::from_value::<CreateMilestoneView>(body);
        let created =
            parse(serde_json::json!({ "title": " Tender ", "due_date": "2026-12-15" })).unwrap();
        assert!(!created.reached);
        assert!(created.validate().is_ok());
        assert_eq!(created.due_date.to_string(), "2026-12-15");
        for blank in ["", "   "] {
            let view = parse(serde_json::json!({ "title": blank, "due_date": "2026-12-15" }));
            assert!(view.unwrap().validate().is_err(), "{blank:?}");
        }
        let long = "a".repeat(256);
        let view = parse(serde_json::json!({ "title": long, "due_date": "2026-12-15" }));
        assert!(view.unwrap().validate().is_err());
        for invalid in [
            serde_json::json!({ "title": "T" }),
            serde_json::json!({ "title": "T", "due_date": "15/12/2026" }),
            serde_json::json!({ "title": "T", "due_date": "2026-02-30" }),
            serde_json::json!({ "title": "T", "due_date": "2026-12-15T10:00:00Z" }),
        ] {
            assert!(parse(invalid.clone()).is_err(), "{invalid}");
        }
    }

    #[test]
    fn mair_1086_a_partial_update_keeps_what_is_absent() {
        let patch: PatchMilestoneView =
            serde_json::from_value(serde_json::json!({ "reached": true })).unwrap();
        assert_eq!(
            (patch.title.as_deref(), patch.due_date, patch.reached),
            (None, None, Some(true))
        );
        assert!(patch.validate().is_ok());
        let blank: PatchMilestoneView =
            serde_json::from_value(serde_json::json!({ "title": " " })).unwrap();
        assert!(blank.validate().is_err());
    }
}
