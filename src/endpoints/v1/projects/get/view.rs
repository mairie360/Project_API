use chrono::{DateTime, Utc};
use mairie360_api_lib::database::db_interface::id_from_sql;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use crate::database::project::get_projects::view::{
    PriorityCounts, ProjectFilters, ProjectView, ProjectsPage, StatusCounts,
};
use crate::endpoints::pagination::{Page, PageParams};
use crate::endpoints::v1::projects::project_id::get::view::TaskPriority;
use crate::endpoints::v1::projects::project_id::users::get::view::User;
use crate::endpoints::validation::{
    check_list, check_opaque, check_optional, Validate, ValidationError, MAX_TITLE_LENGTH,
};

/// Values of `status`, and the status group each one keeps.
const STATUS_FILTERS: [(&str, &str); 4] = [
    ("Active", "active"),
    ("Suspended", "suspended"),
    ("Completed", "completed"),
    ("Error", "other"),
];
/// Values of `priority`, and the stored priority each one keeps. The database has no urgent
/// priority (an `Urgent` task is stored `High`), so no project is ever `Urgent`.
const PRIORITY_FILTERS: [(&str, &str); 3] =
    [("Low", "low"), ("Medium", "medium"), ("High", "high")];

fn names(table: &[(&'static str, &'static str)]) -> Vec<&'static str> {
    table.iter().map(|(name, _)| *name).collect()
}

fn codes(value: Option<&str>, table: &[(&str, &'static str)]) -> Vec<&'static str> {
    value.map_or_else(Vec::new, |value| {
        value
            .split(',')
            .filter_map(|item| {
                table
                    .iter()
                    .find(|(name, _)| *name == item)
                    .map(|(_, code)| *code)
            })
            .collect()
    })
}

/// Page and filters of `GET /api/v1/projects/` (MAIR-474). Every filter is optional and they combine
/// (AND); `summary` and `total` count the projects that match them all.
#[derive(Debug, Default, Clone, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct GetProjectsQuery {
    /// Maximum number of projects returned, from 1 to 500. Defaults to 100. `0` is raised to 1 and a
    /// value above 500 is lowered to 500.
    #[param(minimum = 1, maximum = 500, default = 100, example = 20)]
    limit: Option<u32>,
    /// Number of matching projects skipped before the first one returned. Defaults to 0.
    #[param(minimum = 0, default = 0, example = 0)]
    offset: Option<u32>,
    /// Keeps the projects whose name or description contains this text, case-insensitive (at most
    /// 255 characters, no control character).
    #[param(example = "marché")]
    search: Option<String>,
    /// Keeps the projects with one of these statuses: a comma-separated list of `Active`,
    /// `Suspended`, `Completed` and `Error` (a stored status the API does not interpret).
    #[param(example = "Active,Suspended")]
    status: Option<String>,
    /// Keeps the projects whose priority (the highest priority of their tasks, `Medium` without
    /// task) is one of these: a comma-separated list of `Low`, `Medium` and `High` (an `Urgent`
    /// task is stored `High`).
    #[param(example = "High,Medium")]
    priority: Option<String>,
    /// Keeps the projects whose `due_date` (earliest due date of their tasks) is at or before this
    /// instant (RFC 3339). A project without due date never matches.
    #[param(value_type = Option<String>, format = DateTime, example = "2026-12-31T23:59:59Z")]
    due_before: Option<DateTime<Utc>>,
    /// Keeps the projects whose `due_date` is at or after this instant (RFC 3339). A project without
    /// due date never matches.
    #[param(value_type = Option<String>, format = DateTime, example = "2026-01-01T00:00:00Z")]
    due_after: Option<DateTime<Utc>>,
}

impl GetProjectsQuery {
    pub fn page(&self) -> Page {
        PageParams::new(self.limit, self.offset).page()
    }

    /// The filters, as database codes (call after `validate`).
    pub fn filters(&self) -> ProjectFilters {
        ProjectFilters {
            search: self.search.clone().filter(|search| !search.is_empty()),
            statuses: codes(self.status.as_deref(), &STATUS_FILTERS),
            priorities: codes(self.priority.as_deref(), &PRIORITY_FILTERS),
            due_before: self.due_before,
            due_after: self.due_after,
        }
    }
}

impl Validate for GetProjectsQuery {
    fn validate(&self) -> Result<(), ValidationError> {
        check_optional(self.search.as_deref(), |search| {
            check_opaque("search", search, MAX_TITLE_LENGTH)
        })?;
        check_optional(self.status.as_deref(), |status| {
            check_list("status", status, &names(&STATUS_FILTERS))
        })?;
        check_optional(self.priority.as_deref(), |priority| {
            check_list("priority", priority, &names(&PRIORITY_FILTERS))
        })
    }
}

/// Statut d'un projet : `Active` (en cours), `Suspended` (suspendu) ou `Completed` (terminé).
/// `Error` n'est jamais assignable : il signale à la lecture une valeur en base inconnue.
#[derive(Debug, serde::Serialize, serde::Deserialize, ToSchema)]
pub enum ProjectStatus {
    /// Projet en cours.
    Active,
    Suspended,
    Completed,
    Error,
}

impl From<String> for ProjectStatus {
    fn from(value: String) -> Self {
        match value.as_str() {
            "active" => ProjectStatus::Active,
            "suspended" => ProjectStatus::Suspended,
            "completed" => ProjectStatus::Completed,
            _ => ProjectStatus::Error,
        }
    }
}

impl std::fmt::Display for ProjectStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ProjectStatus::Active => "active",
            ProjectStatus::Suspended => "suspended",
            ProjectStatus::Completed => "completed",
            ProjectStatus::Error => "error",
        };
        f.write_str(s)
    }
}

/// Projet, tel que listé ou consulté.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct ProjetView {
    /// Identifiant du projet, à réutiliser dans `/api/v1/projects/{project_id}/`.
    #[schema(example = 12)]
    pub id: u64,
    /// Nom du projet.
    #[schema(example = "Réfection de la place du marché")]
    pub name: String,
    /// Description du projet. Chaîne vide s'il n'en a pas — jamais `null`.
    #[schema(example = "Travaux de voirie 2026")]
    pub description: String,
    /// Statut courant. `Error` signale une valeur en base que l'API ne sait pas interpréter ;
    /// ce n'est pas un statut assignable.
    pub status: ProjectStatus,
}

impl From<ProjectView> for ProjetView {
    fn from(value: ProjectView) -> Self {
        Self {
            id: id_from_sql(value.id()),
            name: value.title().to_string(),
            description: value.description().unwrap_or_default().to_string(),
            status: value.status().to_string().into(),
        }
    }
}

/// Project of the list, with the aggregates of its tasks and its first members (MAIR-474).
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct ProjectListItemView {
    /// Identifiant du projet, à réutiliser dans `/api/v1/projects/{project_id}/`.
    #[schema(example = 12)]
    pub id: u64,
    /// Nom du projet.
    #[schema(example = "Réfection de la place du marché")]
    pub name: String,
    /// Description du projet. Chaîne vide s'il n'en a pas — jamais `null`.
    #[schema(example = "Travaux de voirie 2026")]
    pub description: String,
    /// Statut courant. `Error` signale une valeur en base que l'API ne sait pas interpréter.
    pub status: ProjectStatus,
    /// Number of tasks of the project.
    #[schema(example = 12)]
    pub tasks_total: u64,
    /// Number of its tasks whose status is `Completed`.
    #[schema(example = 4)]
    pub tasks_completed: u64,
    /// Highest priority of its tasks; `Medium` (the default priority of a task) when it has none.
    /// Never `Urgent`: an `Urgent` task is stored `High`.
    pub priority: TaskPriority,
    /// Earliest due date of its tasks, or `null` when none of them has one.
    #[schema(value_type = Option<String>, format = DateTime, example = "2026-10-15T00:00:00Z")]
    pub due_date: Option<DateTime<Utc>>,
    /// Its first 5 members, by last name, first name then id (`GET …/{project_id}/users/` lists them
    /// all).
    pub members: Vec<User>,
    /// Number of its members.
    #[schema(example = 7)]
    pub members_total: u64,
}

fn count(value: i64) -> u64 {
    u64::try_from(value).unwrap_or(0)
}

impl From<ProjectView> for ProjectListItemView {
    fn from(value: ProjectView) -> Self {
        Self {
            id: id_from_sql(value.id()),
            name: value.title().to_string(),
            description: value.description().unwrap_or_default().to_string(),
            status: value.status().to_string().into(),
            tasks_total: count(value.tasks_total()),
            tasks_completed: count(value.tasks_completed()),
            priority: value.priority().to_string().into(),
            due_date: value.due_date(),
            members: value
                .members()
                .iter()
                .map(|member| User {
                    id: id_from_sql(member.id),
                    name: member.name.clone(),
                })
                .collect(),
            members_total: count(value.members_total()),
        }
    }
}

/// Number of matching projects per status.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct ProjectStatusCountsView {
    #[schema(example = 3)]
    pub active: u64,
    #[schema(example = 1)]
    pub suspended: u64,
    #[schema(example = 2)]
    pub completed: u64,
    /// Projects whose stored status the API does not interpret (listed with the `Error` status).
    #[schema(example = 0)]
    pub other: u64,
}

impl From<StatusCounts> for ProjectStatusCountsView {
    fn from(value: StatusCounts) -> Self {
        Self {
            active: count(value.active),
            suspended: count(value.suspended),
            completed: count(value.completed),
            other: count(value.other),
        }
    }
}

/// Number of matching projects per priority (the highest priority of their tasks).
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct ProjectPriorityCountsView {
    #[schema(example = 1)]
    pub low: u64,
    #[schema(example = 3)]
    pub medium: u64,
    #[schema(example = 2)]
    pub high: u64,
}

impl From<PriorityCounts> for ProjectPriorityCountsView {
    fn from(value: PriorityCounts) -> Self {
        Self {
            low: count(value.low),
            medium: count(value.medium),
            high: count(value.high),
        }
    }
}

/// Counts over every project matching the filters, whatever the page.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct ProjectsSummaryView {
    pub by_status: ProjectStatusCountsView,
    pub by_priority: ProjectPriorityCountsView,
}

/// Projets visibles par l'utilisateur connecté.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetProjectsResultView {
    /// Projects of the page, newest first. Empty if no visible project matches the filters or the
    /// page is past the end.
    pub projects: Vec<ProjectListItemView>,
    /// Number of visible projects matching the filters, whatever the page.
    #[schema(example = 2)]
    pub total: u64,
    pub summary: ProjectsSummaryView,
}

impl From<ProjectsPage> for GetProjectsResultView {
    fn from(page: ProjectsPage) -> Self {
        Self {
            projects: page.items.into_iter().map(Into::into).collect(),
            total: count(page.total),
            summary: ProjectsSummaryView {
                by_status: page.by_status.into(),
                by_priority: page.by_priority.into(),
            },
        }
    }
}
