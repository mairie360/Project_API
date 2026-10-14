use mairie360_api_lib::database::db_interface::id_from_sql;
use utoipa::ToSchema;

use crate::database::statuses::view::{ProjectStatusRow, StatusCategory};
use crate::endpoints::validation::{
    check_color, check_label, check_optional, Validate, ValidationError, MAX_STATUS_NAME_LENGTH,
};

/// A task status of a project (MAIR-566).
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct ProjectStatusView {
    /// Id of the status, the `status_id` of the tasks in it.
    #[schema(example = 31)]
    pub id: u64,
    /// Name shown for the status (a Kanban column).
    #[schema(example = "À valider")]
    pub name: String,
    /// Colour of the status, `#RRGGBB`.
    #[schema(example = "#D97706")]
    pub color: String,
    /// Category of the status: what it means for the archiving, the progress and the pending task
    /// lists. `Completed` tasks are archived.
    pub category: StatusCategory,
    /// Position among the project's statuses, from 0.
    #[schema(example = 2)]
    pub position: u32,
    /// Tasks of the project in this status, archived ones included.
    #[schema(example = 4)]
    pub task_count: u64,
}

impl ProjectStatusView {
    /// The view of a row; `None` for a category the API does not use.
    pub fn from_row(row: ProjectStatusRow) -> Option<Self> {
        Some(Self {
            id: id_from_sql(row.id),
            name: row.name,
            color: row.color,
            category: StatusCategory::from_stored(&row.category)?,
            position: u32::try_from(row.position).unwrap_or_default(),
            task_count: u64::try_from(row.task_count).unwrap_or_default(),
        })
    }
}

/// The task statuses of a project, by position.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetProjectStatusesResultView {
    pub statuses: Vec<ProjectStatusView>,
}

/// A status to add to a project.
#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct CreateProjectStatusView {
    /// Name, 1 to 100 characters, not blank, no control character, unique in the project
    /// whatever the case.
    #[schema(min_length = 1, max_length = 100, example = "À valider")]
    pub name: String,
    /// Colour, `#RRGGBB`. Absent: `#64748B`.
    #[schema(example = "#D97706")]
    pub color: Option<String>,
    /// Category of the status.
    pub category: StatusCategory,
    /// Position among the project's statuses, from 0; the next ones move down. Absent or past
    /// the end: last.
    #[schema(example = 2)]
    pub position: Option<u32>,
}

impl Validate for CreateProjectStatusView {
    fn validate(&self) -> Result<(), ValidationError> {
        check_label("name", &self.name, MAX_STATUS_NAME_LENGTH)?;
        check_optional(self.color.as_deref(), |color| check_color("color", color))
    }
}

/// Status added to the project.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct CreateProjectStatusResultView {
    #[schema(example = 31)]
    pub status_id: u64,
}

/// Partial update of a status: an absent field is left unchanged.
#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct PatchProjectStatusView {
    /// New name, same rules as on creation.
    #[schema(min_length = 1, max_length = 100, example = "En validation")]
    pub name: Option<String>,
    /// New colour, `#RRGGBB`.
    #[schema(example = "#7C3AED")]
    pub color: Option<String>,
    /// New category: the tasks in this status move to it (a `Completed` status archives them).
    pub category: Option<StatusCategory>,
    /// New position, from 0; the statuses in between shift. Past the end: last.
    #[schema(example = 0)]
    pub position: Option<u32>,
}

impl Validate for PatchProjectStatusView {
    fn validate(&self) -> Result<(), ValidationError> {
        check_optional(self.name.as_deref(), |name| {
            check_label("name", name, MAX_STATUS_NAME_LENGTH)
        })?;
        check_optional(self.color.as_deref(), |color| check_color("color", color))
    }
}

/// Query of `DELETE …/statuses/{status_id}`.
#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct DeleteProjectStatusQuery {
    /// Another status of the project to move the tasks of the deleted one to. Required when the
    /// status holds tasks.
    #[param(example = 30)]
    pub move_to: Option<u64>,
}
