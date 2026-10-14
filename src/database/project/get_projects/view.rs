use chrono::{DateTime, Utc};
use mairie360_api_lib::database::db_interface::{
    id_from_sql, id_to_sql, ApiRequestDto, QueryParam,
};

use crate::database::users::get_project_users::view::ProjectMemberRow;

/// Filters of the projects list (MAIR-474). Every filter is optional; an empty one keeps every
/// project. The values are the database codes, already checked by the endpoint.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProjectFilters {
    /// Case-insensitive substring of the title or the description.
    pub search: Option<String>,
    /// Status groups kept: `active`, `suspended`, `completed`, `other` (any other stored value).
    pub statuses: Vec<&'static str>,
    /// Priorities kept (`low`, `medium`, `high`): the highest priority of the project's tasks,
    /// `medium` (the default priority of a task) when it has none. The database has no `urgent`.
    pub priorities: Vec<&'static str>,
    /// Keeps the projects whose earliest task due date is at or before this instant.
    pub due_before: Option<DateTime<Utc>>,
    /// Keeps the projects whose earliest task due date is at or after this instant.
    pub due_after: Option<DateTime<Utc>>,
}

/// One page of the projects visible to the user (see `project_visible_to_user_sql!`), newest first,
/// with the aggregates of their tasks and a summary of every matching project (MAIR-474): the BFF used
/// to read every task of every visible project to compute them. Read with
/// `fetch_one::<ProjectsPage, _>` (or `PagedRows<ProjectView>`, which ignores the summary).
///
/// The tasks are aggregated per project through `idx_tasks_project_id`, once, in a materialized
/// CTE; the members are only read for the rows of the page. On the volume seed (5 000 projects,
/// 50 000 tasks) an Admin page costs about 30 ms, an agent's 2 ms.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetProjectsQueryView {
    params: Vec<QueryParam>,
}

fn optional_text(value: Option<String>) -> QueryParam {
    QueryParam::Text(value.unwrap_or_default())
}

impl GetProjectsQueryView {
    /// Every visible project, unfiltered.
    pub fn new(user_id: u64, limit: u32, offset: u32) -> Self {
        Self::filtered(user_id, &ProjectFilters::default(), limit, offset)
    }

    pub fn filtered(user_id: u64, filters: &ProjectFilters, limit: u32, offset: u32) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(user_id)),
                optional_text(filters.search.clone()),
                QueryParam::Text(filters.statuses.join(",")),
                QueryParam::Text(filters.priorities.join(",")),
                optional_text(filters.due_before.map(|d| d.to_rfc3339())),
                optional_text(filters.due_after.map(|d| d.to_rfc3339())),
                QueryParam::I64(i64::from(limit)),
                QueryParam::I64(i64::from(offset)),
            ],
        }
    }

    pub fn user_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }
}

impl ApiRequestDto for GetProjectsQueryView {
    fn query_sql(&self) -> &'static str {
        concat!(
            "WITH matched AS MATERIALIZED ( \
                SELECT p.id, p.title, p.description, p.status, p.created_at, \
                       COALESCE(a.tasks_total, 0) AS tasks_total, \
                       COALESCE(a.tasks_completed, 0) AS tasks_completed, \
                       COALESCE(a.top_priority::text, 'medium') AS priority, \
                       a.next_due_date AS due_date \
                FROM projects p \
                LEFT JOIN LATERAL ( \
                    SELECT count(*) AS tasks_total, \
                           count(*) FILTER (WHERE t.status = 'completed') AS tasks_completed, \
                           max(t.priority) AS top_priority, min(t.due_date) AS next_due_date \
                    FROM tasks t WHERE t.project_id = p.id) a ON true \
                WHERE ",
            crate::project_visible_to_user_sql!(),
            " AND ($2 = '' OR strpos(lower(p.title), lower($2)) > 0 \
                     OR strpos(lower(COALESCE(p.description, '')), lower($2)) > 0) \
                  AND ($3 = '' OR (CASE WHEN p.status::text IN ('active', 'suspended', 'completed') \
                                   THEN p.status::text ELSE 'other' END) = ANY (string_to_array($3, ','))) \
             ), filtered AS MATERIALIZED ( \
                SELECT * FROM matched m \
                WHERE ($4 = '' OR m.priority = ANY (string_to_array($4, ','))) \
                  AND (NULLIF($5, '')::timestamptz IS NULL \
                       OR m.due_date <= NULLIF($5, '')::timestamptz AT TIME ZONE 'UTC') \
                  AND (NULLIF($6, '')::timestamptz IS NULL \
                       OR m.due_date >= NULLIF($6, '')::timestamptz AT TIME ZONE 'UTC') \
             ), page AS ( \
                SELECT * FROM filtered \
                ORDER BY created_at DESC NULLS LAST, id DESC LIMIT $7 OFFSET $8 \
             ) \
             SELECT jsonb_build_object( \
                'total', (SELECT count(*) FROM filtered), \
                'by_status', (SELECT jsonb_build_object( \
                    'active', count(*) FILTER (WHERE status = 'active'), \
                    'suspended', count(*) FILTER (WHERE status = 'suspended'), \
                    'completed', count(*) FILTER (WHERE status = 'completed'), \
                    'other', count(*) FILTER (WHERE status::text NOT IN ('active', 'suspended', 'completed'))) \
                    FROM filtered), \
                'by_priority', (SELECT jsonb_build_object( \
                    'low', count(*) FILTER (WHERE priority = 'low'), \
                    'medium', count(*) FILTER (WHERE priority = 'medium'), \
                    'high', count(*) FILTER (WHERE priority = 'high')) \
                    FROM filtered), \
                'items', COALESCE((SELECT jsonb_agg(to_jsonb(r) ORDER BY r.created_at DESC NULLS LAST, r.id DESC) \
                    FROM ( \
                        SELECT pg.id, pg.title, pg.description, pg.status, pg.created_at, \
                               pg.tasks_total, pg.tasks_completed, pg.priority, \
                               pg.due_date AT TIME ZONE 'UTC' AS due_date, \
                               (SELECT count(*) FROM project_members pm WHERE pm.project_id = pg.id) \
                                   AS members_total, \
                               COALESCE((SELECT jsonb_agg(jsonb_build_object('id', m.id, 'name', m.name) \
                                                          ORDER BY m.last_name, m.first_name, m.id) \
                                         FROM (SELECT u.id, u.first_name, u.last_name, \
                                                      NULLIF(concat_ws(' ', u.first_name, u.last_name), '') AS name \
                                               FROM project_members pm JOIN users u ON u.id = pm.user_id \
                                               WHERE pm.project_id = pg.id \
                                               ORDER BY u.last_name, u.first_name, u.id LIMIT 5) m), \
                                        '[]'::jsonb) AS members \
                        FROM page pg) r), '[]'::jsonb))"
        )
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Number of matching projects per status group.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StatusCounts {
    pub active: i64,
    pub suspended: i64,
    pub completed: i64,
    /// Any other stored status (`todo`, `review`), read as `Error` by the API.
    pub other: i64,
}

/// Number of matching projects per priority (the highest priority of their tasks).
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PriorityCounts {
    pub low: i64,
    pub medium: i64,
    pub high: i64,
}

/// A page of `GetProjectsQueryView`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProjectsPage {
    /// Number of matching projects, whatever the page.
    pub total: i64,
    pub by_status: StatusCounts,
    pub by_priority: PriorityCounts,
    pub items: Vec<ProjectView>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProjectView {
    id: i32,
    title: String,
    description: Option<String>,
    status: String,
    #[serde(default)]
    tasks_total: i64,
    #[serde(default)]
    tasks_completed: i64,
    #[serde(default = "default_priority")]
    priority: String,
    #[serde(default)]
    due_date: Option<DateTime<Utc>>,
    #[serde(default)]
    members: Vec<ProjectMemberRow>,
    #[serde(default)]
    members_total: i64,
}

fn default_priority() -> String {
    "medium".to_string()
}

impl ProjectView {
    pub fn new(id: i32, title: &str, description: Option<&str>, status: &str) -> Self {
        Self {
            id,
            title: title.to_string(),
            description: description.map(str::to_string),
            status: status.to_string(),
            tasks_total: 0,
            tasks_completed: 0,
            priority: default_priority(),
            due_date: None,
            members: Vec::new(),
            members_total: 0,
        }
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    /// Number of tasks of the project.
    pub fn tasks_total(&self) -> i64 {
        self.tasks_total
    }

    /// Number of its tasks whose status is `completed`.
    pub fn tasks_completed(&self) -> i64 {
        self.tasks_completed
    }

    /// Highest priority of its tasks, `medium` when it has none.
    pub fn priority(&self) -> &str {
        &self.priority
    }

    /// Earliest due date of its tasks, `None` when none has one.
    pub fn due_date(&self) -> Option<DateTime<Utc>> {
        self.due_date
    }

    /// Its first members, by name.
    pub fn members(&self) -> &[ProjectMemberRow] {
        &self.members
    }

    /// Number of its members.
    pub fn members_total(&self) -> i64 {
        self.members_total
    }
}
