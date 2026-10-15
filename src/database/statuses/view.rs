use mairie360_api_lib::database::db_interface::{id_to_sql, ApiRequestDto, QueryParam};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Category of a task status (MAIR-566), whatever its name: it drives the archiving (`Completed`
/// tasks are archived), the progress and the pending task lists. Every project and template keeps
/// at least one status of each category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum StatusCategory {
    Todo,
    InProgress,
    Completed,
}

impl StatusCategory {
    /// Every category, in workflow order.
    pub const ALL: [StatusCategory; 3] = [
        StatusCategory::Todo,
        StatusCategory::InProgress,
        StatusCategory::Completed,
    ];

    /// The `task_status` value stored in the database.
    pub fn stored(self) -> &'static str {
        match self {
            StatusCategory::Todo => "todo",
            StatusCategory::InProgress => "in_progress",
            StatusCategory::Completed => "completed",
        }
    }

    /// The category of a stored `task_status`, `None` for a value the API does not use.
    pub fn from_stored(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|category| category.stored() == value)
    }
}

/// Colour of a status written without one.
pub const DEFAULT_STATUS_COLOR: &str = "#64748B";

/// A status of a project, as read by [`ListProjectStatusesQueryView`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStatusRow {
    pub id: i32,
    pub name: String,
    pub color: String,
    pub category: String,
    pub position: i32,
    /// Tasks of the project in this status, archived ones included.
    pub task_count: i64,
}

/// The statuses of a project, by position. Read with `fetch_one::<Vec<ProjectStatusRow>, _>`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListProjectStatusesQueryView {
    params: Vec<QueryParam>,
}

impl ListProjectStatusesQueryView {
    pub fn new(project_id: u64) -> Self {
        Self {
            params: vec![QueryParam::I32(id_to_sql(project_id))],
        }
    }
}

impl ApiRequestDto for ListProjectStatusesQueryView {
    fn query_sql(&self) -> &'static str {
        "SELECT COALESCE(jsonb_agg(jsonb_build_object( \
            'id', s.id, 'name', s.name, 'color', s.color, 'category', s.category, \
            'position', s.position, \
            'task_count', (SELECT count(*) FROM tasks t WHERE t.status_id = s.id)) \
            ORDER BY s.position, s.id), '[]'::jsonb) \
         FROM project_task_statuses s WHERE s.project_id = $1"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Adds a status to a project at `position` (the end when absent or past it), moving the next
/// ones down, and returns its id. A name already used in the project (whatever the case) is a
/// unique violation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectStatusQueryView {
    params: Vec<QueryParam>,
}

impl CreateProjectStatusQueryView {
    pub fn new(
        project_id: u64,
        name: &str,
        color: Option<&str>,
        category: StatusCategory,
        position: Option<u32>,
    ) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(project_id)),
                QueryParam::Text(name.to_string()),
                QueryParam::Text(color.unwrap_or(DEFAULT_STATUS_COLOR).to_string()),
                QueryParam::Text(category.stored().to_string()),
                QueryParam::OptionI32(position.map(position_to_sql)),
            ],
        }
    }
}

impl ApiRequestDto for CreateProjectStatusQueryView {
    fn query_sql(&self) -> &'static str {
        // Positions stay 0..n-1: the new one takes `position` (at most n), the next ones move down.
        // uq_project_task_statuses_position is deferred, so the shift and the insert can overlap.
        "WITH target AS ( \
            SELECT LEAST(COALESCE($5::int, 2147483647), \
                         (SELECT count(*) FROM project_task_statuses WHERE project_id = $1)::int) AS pos \
         ), shifted AS ( \
            UPDATE project_task_statuses SET position = position + 1 \
            WHERE project_id = $1 AND position >= (SELECT pos FROM target) RETURNING id \
         ) \
         INSERT INTO project_task_statuses (project_id, name, color, category, position) \
         SELECT $1, $2, $3, $4::task_status, pos FROM target \
         RETURNING id"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }

    fn invalidated_read_caches(&self) -> &'static [&'static str] {
        crate::database::project::PROJECTS_READ_CACHES
    }
}

/// Changes requested on a project status. A `None` field is kept.
#[derive(Debug, Clone, Default)]
pub struct StatusChanges<'a> {
    pub name: Option<&'a str>,
    pub color: Option<&'a str>,
    pub category: Option<StatusCategory>,
    pub position: Option<u32>,
}

/// [`UpdateProjectStatusQueryView`]: the status was updated.
pub const STATUS_UPDATED: i32 = 1;
/// No such status in this project: nothing written.
pub const STATUS_NOT_FOUND: i32 = 0;
/// The new category would leave the former one without any status in the project: nothing
/// written.
pub const STATUS_LAST_OF_CATEGORY: i32 = -1;

/// Renames, recolours, moves (`position`, the next ones shifting) or recategorizes a status of a
/// project; changing the category moves its tasks to it (database trigger). Returns
/// [`STATUS_UPDATED`], [`STATUS_NOT_FOUND`] or [`STATUS_LAST_OF_CATEGORY`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProjectStatusQueryView {
    params: Vec<QueryParam>,
}

impl UpdateProjectStatusQueryView {
    pub fn new(project_id: u64, status_id: u64, changes: StatusChanges<'_>) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(project_id)),
                QueryParam::I32(id_to_sql(status_id)),
                QueryParam::Text(changes.name.unwrap_or_default().to_string()),
                QueryParam::Text(changes.color.unwrap_or_default().to_string()),
                QueryParam::Text(
                    changes
                        .category
                        .map(|c| c.stored().to_string())
                        .unwrap_or_default(),
                ),
                QueryParam::OptionI32(changes.position.map(position_to_sql)),
            ],
        }
    }
}

impl ApiRequestDto for UpdateProjectStatusQueryView {
    fn query_sql(&self) -> &'static str {
        "WITH cur AS ( \
            SELECT id, position AS old_pos, category FROM project_task_statuses \
            WHERE id = $2 AND project_id = $1 \
         ), refused AS ( \
            SELECT NULLIF($5, '')::task_status IS NOT NULL \
                   AND NULLIF($5, '')::task_status <> cur.category \
                   AND NOT EXISTS (SELECT 1 FROM project_task_statuses o \
                                   WHERE o.project_id = $1 AND o.category = cur.category \
                                     AND o.id <> cur.id) AS last_of_category \
            FROM cur \
         ), target AS ( \
            SELECT cur.old_pos, \
                   CASE WHEN $6::int IS NULL THEN cur.old_pos \
                        ELSE LEAST($6::int, (SELECT count(*) - 1 FROM project_task_statuses \
                                             WHERE project_id = $1)::int) END AS pos \
            FROM cur, refused WHERE NOT refused.last_of_category \
         ), shifted AS ( \
            UPDATE project_task_statuses o SET position = o.position + CASE \
                WHEN t.pos < t.old_pos AND o.position >= t.pos AND o.position < t.old_pos THEN 1 \
                WHEN t.pos > t.old_pos AND o.position <= t.pos AND o.position > t.old_pos THEN -1 \
                ELSE 0 END \
            FROM target t \
            WHERE o.project_id = $1 AND o.id <> $2 AND t.pos <> t.old_pos RETURNING o.id \
         ), updated AS ( \
            UPDATE project_task_statuses s SET \
                name = COALESCE(NULLIF($3, ''), s.name), \
                color = COALESCE(NULLIF($4, ''), s.color), \
                category = COALESCE(NULLIF($5, '')::task_status, s.category), \
                position = t.pos \
            FROM target t WHERE s.id = $2 AND s.project_id = $1 RETURNING s.id \
         ) \
         SELECT CASE WHEN NOT EXISTS (SELECT 1 FROM cur) THEN 0 \
                     WHEN EXISTS (SELECT 1 FROM refused WHERE last_of_category) THEN -1 \
                     ELSE 1 END"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }

    fn invalidated_read_caches(&self) -> &'static [&'static str] {
        crate::database::project::PROJECTS_READ_CACHES
    }
}

/// [`CheckStatusDeletionQueryView`]: the status can be deleted (after moving its tasks).
pub const DELETION_ALLOWED: i32 = 1;
/// No such status in this project.
pub const DELETION_NOT_FOUND: i32 = 0;
/// The status holds tasks and no status to move them to was given.
pub const DELETION_TASKS_TO_MOVE: i32 = -1;
/// The status to move the tasks to is not another status of this project.
pub const DELETION_INVALID_TARGET: i32 = -2;
/// It is the last status of its category in the project.
pub const DELETION_LAST_OF_CATEGORY: i32 = -3;

/// Says whether a status of a project can be deleted, its tasks going to `move_to`. Returns one
/// of the `DELETION_*` codes. Run in the write transaction, the project row locked.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckStatusDeletionQueryView {
    params: Vec<QueryParam>,
}

impl CheckStatusDeletionQueryView {
    pub fn new(project_id: u64, status_id: u64, move_to: Option<u64>) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(project_id)),
                QueryParam::I32(id_to_sql(status_id)),
                QueryParam::OptionI32(move_to.map(id_to_sql)),
            ],
        }
    }
}

impl ApiRequestDto for CheckStatusDeletionQueryView {
    fn query_sql(&self) -> &'static str {
        "WITH cur AS ( \
            SELECT id, category FROM project_task_statuses WHERE id = $2 AND project_id = $1 \
         ) \
         SELECT CASE \
            WHEN NOT EXISTS (SELECT 1 FROM cur) THEN 0 \
            WHEN $3::int IS NOT NULL AND NOT EXISTS ( \
                SELECT 1 FROM project_task_statuses WHERE id = $3 AND project_id = $1 AND id <> $2) \
                THEN -2 \
            WHEN $3::int IS NULL AND EXISTS (SELECT 1 FROM tasks WHERE status_id = $2) THEN -1 \
            WHEN NOT EXISTS (SELECT 1 FROM project_task_statuses o, cur \
                             WHERE o.project_id = $1 AND o.category = cur.category AND o.id <> cur.id) \
                THEN -3 \
            ELSE 1 END"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Moves every task of a status to another status of the same project, signed by `moved_by`
/// (the history logs a `status_changed` entry per task).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveStatusTasksQueryView {
    params: Vec<QueryParam>,
}

impl MoveStatusTasksQueryView {
    pub fn new(project_id: u64, from_status: u64, to_status: u64, moved_by: u64) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(project_id)),
                QueryParam::I32(id_to_sql(from_status)),
                QueryParam::I32(id_to_sql(to_status)),
                QueryParam::I32(id_to_sql(moved_by)),
            ],
        }
    }
}

impl ApiRequestDto for MoveStatusTasksQueryView {
    fn query_sql(&self) -> &'static str {
        "UPDATE tasks SET status_id = $3, updated_by = $4, updated_at = CURRENT_TIMESTAMP \
         WHERE project_id = $1 AND status_id = $2"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }

    fn invalidated_read_caches(&self) -> &'static [&'static str] {
        crate::database::project::PROJECTS_READ_CACHES
    }
}

/// Deletes a status without tasks from a project and moves the next ones up.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteProjectStatusQueryView {
    params: Vec<QueryParam>,
}

impl DeleteProjectStatusQueryView {
    pub fn new(project_id: u64, status_id: u64) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(project_id)),
                QueryParam::I32(id_to_sql(status_id)),
            ],
        }
    }
}

impl ApiRequestDto for DeleteProjectStatusQueryView {
    fn query_sql(&self) -> &'static str {
        "WITH deleted AS ( \
            DELETE FROM project_task_statuses WHERE id = $2 AND project_id = $1 RETURNING position \
         ) \
         UPDATE project_task_statuses SET position = position - 1 \
         WHERE project_id = $1 AND position > (SELECT position FROM deleted)"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }

    fn invalidated_read_caches(&self) -> &'static [&'static str] {
        crate::database::project::PROJECTS_READ_CACHES
    }
}

/// A position from the API (`u32`) as an SQL `int`, capped: a position past the end means "last".
pub fn position_to_sql(position: u32) -> i32 {
    i32::try_from(position).unwrap_or(i32::MAX)
}
