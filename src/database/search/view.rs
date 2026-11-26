use mairie360_api_lib::database::db_interface::{id_to_sql, ApiRequestDto, QueryParam};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use utoipa::ToSchema;

/// What a global search result is.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GlobalSearchKind {
    /// A project; `id` is reusable in `GET /api/v1/projects/{id}`.
    Project,
    /// A task; `parent_id` is its project.
    Task,
}

/// One result of the global search.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct GlobalSearchItem {
    pub kind: GlobalSearchKind,
    /// Id of the project or of the task.
    #[schema(example = 42)]
    pub id: i64,
    /// Title of the project or of the task.
    #[schema(example = "Jules Ferry school renovation")]
    pub title: String,
    /// First 120 characters of the description, `null` without one.
    #[schema(example = "New windows and attic insulation")]
    pub excerpt: Option<String>,
    /// Project of a task; `null` for a project.
    #[schema(example = json!(null))]
    pub parent_id: Option<i64>,
    /// Last change of a task, creation of a project (UTC, RFC 3339).
    #[schema(format = DateTime, example = "2026-11-20T09:30:00Z")]
    pub updated_at: String,
}

/// Projects and tasks matching a global search (`GET /api/v1/search`), among the projects the
/// caller sees (`project_visible_to_user_sql!`) and their tasks. A project matches on its title
/// or description, a task on its title, through the trigram indexes. Titles starting with the
/// term come first, then the most recent.
#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalSearchQueryView {
    params: Vec<QueryParam>,
}

impl GlobalSearchQueryView {
    /// `pattern` is the term already escaped for `LIKE` (see `like_escape`), `limit` bounded by
    /// the endpoint.
    pub fn new(user_id: u64, pattern: &str, limit: u64) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(user_id)),
                QueryParam::Text(pattern.to_string()),
                QueryParam::I64(i64::try_from(limit).unwrap_or(i64::MAX)),
            ],
        }
    }
}

impl ApiRequestDto for GlobalSearchQueryView {
    fn query_sql(&self) -> &'static str {
        concat!(
            "WITH visible AS MATERIALIZED ( \
                SELECT p.id, p.title, p.description, p.created_at FROM projects p WHERE ",
            crate::project_visible_to_user_sql!(),
            " ), hits AS ( \
                SELECT 'project' AS kind, v.id, v.title, v.description, NULL::int AS parent_id, \
                       v.created_at AS changed_at \
                FROM visible v \
                WHERE v.title ILIKE '%' || $2 || '%' OR v.description ILIKE '%' || $2 || '%' \
                UNION ALL \
                SELECT 'task', t.id, t.title, t.description, t.project_id, \
                       COALESCE(t.updated_at, t.created_at) \
                FROM tasks t JOIN visible v ON v.id = t.project_id \
                WHERE t.title ILIKE '%' || $2 || '%' \
             ) \
             SELECT jsonb_build_object( \
                'kind', h.kind, 'id', h.id, 'title', h.title, \
                'excerpt', NULLIF(left(btrim(COALESCE(h.description, '')), 120), ''), \
                'parent_id', h.parent_id, \
                'updated_at', to_char(h.changed_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"')) \
             FROM hits h \
             ORDER BY (h.title ILIKE $2 || '%') DESC, h.changed_at DESC NULLS LAST, \
                      h.kind, h.id DESC \
             LIMIT $3"
        )
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

impl Display for GlobalSearchQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The search term is not logged.
        write!(f, "GlobalSearchQueryView: limit={:?}", self.params[2])
    }
}
