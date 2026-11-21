use chrono::{DateTime, NaiveDate, Utc};
use mairie360_api_lib::database::db_interface::{id_to_sql, ApiRequestDto, QueryParam};
use serde::{Deserialize, Serialize};

// The milestones are not shown by `GET /projects/` (only by the project detail, which is not
// cached), so no write view here invalidates `PROJECTS_READ_CACHES`.

/// Columns of a milestone row `m`, as read back by every view of this module.
macro_rules! milestone_columns_sql {
    () => {
        "m.id, m.project_id, m.title, m.due_date, m.reached, m.created_by, m.updated_by, \
         m.created_at, m.updated_at"
    };
}

/// A milestone of a project, not deleted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MilestoneRow {
    pub id: i32,
    pub project_id: i32,
    pub title: String,
    pub due_date: NaiveDate,
    pub reached: bool,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// One page of the milestones of a project that are not deleted, by due date then id. Read with
/// `fetch_one::<PagedRows<MilestoneRow>, _>`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetProjectMilestonesQueryView {
    params: Vec<QueryParam>,
}

impl GetProjectMilestonesQueryView {
    pub fn new(project_id: u64, limit: u32, offset: u32) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(project_id)),
                QueryParam::I64(i64::from(limit)),
                QueryParam::I64(i64::from(offset)),
            ],
        }
    }
}

impl ApiRequestDto for GetProjectMilestonesQueryView {
    fn query_sql(&self) -> &'static str {
        concat!(
            crate::paged_rows_sql!("$2", "$3"),
            " FROM (SELECT ",
            milestone_columns_sql!(),
            ", row_number() OVER (ORDER BY m.due_date, m.id) AS rn \
                FROM project_milestones m \
                WHERE m.project_id = $1 AND m.deleted_at IS NULL) t"
        )
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Adds a milestone to a project, signed by its author, and returns it. Read with
/// `fetch_one::<MilestoneRow, _>`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMilestoneQueryView {
    params: Vec<QueryParam>,
}

impl CreateMilestoneQueryView {
    pub fn new(
        project_id: u64,
        created_by: u64,
        title: &str,
        due_date: NaiveDate,
        reached: bool,
    ) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(project_id)),
                QueryParam::Text(title.to_string()),
                QueryParam::Text(due_date.format("%Y-%m-%d").to_string()),
                QueryParam::Bool(reached),
                QueryParam::I32(id_to_sql(created_by)),
            ],
        }
    }
}

impl ApiRequestDto for CreateMilestoneQueryView {
    fn query_sql(&self) -> &'static str {
        concat!(
            "WITH m AS ( \
                INSERT INTO project_milestones (project_id, title, due_date, reached, \
                                                created_by, updated_by, created_at, updated_at) \
                VALUES ($1, $2, $3::date, $4, $5, $5, NOW(), NOW()) \
                RETURNING * \
             ) SELECT to_jsonb(t) FROM (SELECT ",
            milestone_columns_sql!(),
            " FROM m) t"
        )
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Changes asked on a milestone; a `None` field is kept.
#[derive(Debug, Clone, Default)]
pub struct MilestoneChanges<'a> {
    pub title: Option<&'a str>,
    pub due_date: Option<NaiveDate>,
    pub reached: Option<bool>,
}

/// Updates a milestone of a project that is not deleted, signed by the caller (`updated_by`), and
/// returns it. Read with `fetch_all::<MilestoneRow, _>`: no row when the milestone is not one of
/// the project's, or is deleted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchMilestoneQueryView {
    params: Vec<QueryParam>,
}

impl PatchMilestoneQueryView {
    pub fn new(
        project_id: u64,
        milestone_id: u64,
        updated_by: u64,
        changes: &MilestoneChanges<'_>,
    ) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(project_id)),
                QueryParam::I32(id_to_sql(milestone_id)),
                QueryParam::Text(changes.title.unwrap_or_default().to_string()),
                QueryParam::Text(
                    changes
                        .due_date
                        .map(|date| date.format("%Y-%m-%d").to_string())
                        .unwrap_or_default(),
                ),
                QueryParam::Bool(changes.reached.is_some()),
                QueryParam::Bool(changes.reached.unwrap_or(false)),
                QueryParam::I32(id_to_sql(updated_by)),
            ],
        }
    }
}

impl ApiRequestDto for PatchMilestoneQueryView {
    fn query_sql(&self) -> &'static str {
        concat!(
            "WITH m AS ( \
                UPDATE project_milestones SET \
                    title = COALESCE(NULLIF($3, ''), title), \
                    due_date = COALESCE(NULLIF($4, '')::date, due_date), \
                    reached = CASE WHEN $5 THEN $6 ELSE reached END, \
                    updated_by = $7, \
                    updated_at = NOW() \
                WHERE id = $2 AND project_id = $1 AND deleted_at IS NULL \
                RETURNING * \
             ) SELECT to_jsonb(t) FROM (SELECT ",
            milestone_columns_sql!(),
            " FROM m) t"
        )
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Soft-deletes a milestone of a project (`deleted_at`), signed by the caller. Read with
/// `fetch_scalar::<bool, _>`: `false` when the milestone is not one of the project's or is already
/// deleted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteMilestoneQueryView {
    params: Vec<QueryParam>,
}

impl DeleteMilestoneQueryView {
    pub fn new(project_id: u64, milestone_id: u64, deleted_by: u64) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(project_id)),
                QueryParam::I32(id_to_sql(milestone_id)),
                QueryParam::I32(id_to_sql(deleted_by)),
            ],
        }
    }
}

impl ApiRequestDto for DeleteMilestoneQueryView {
    fn query_sql(&self) -> &'static str {
        "WITH deleted AS ( \
            UPDATE project_milestones \
            SET deleted_at = NOW(), updated_by = $3, updated_at = NOW() \
            WHERE id = $2 AND project_id = $1 AND deleted_at IS NULL \
            RETURNING id \
         ) SELECT EXISTS (SELECT 1 FROM deleted)"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
