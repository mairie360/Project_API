use crate::database::search::view::{GlobalSearchQueryView, SearchItem};
use crate::endpoints::db_error::log_db_error;
use crate::endpoints::v1::search::view::{like_escape, SearchQuery, SearchResultView};
use crate::endpoints::validation::ValidatedQuery;
use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchError {
    DatabaseError,
}

impl std::fmt::Display for SearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "An error occurred while accessing the database.")
    }
}

impl ResponseError for SearchError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_global_search(
    state: web::Data<AppState>,
    caller_id: u64,
    query: SearchQuery,
) -> Result<SearchResultView, SearchError> {
    let view = GlobalSearchQueryView::new(caller_id, &like_escape(query.term()), query.limit());
    let items: Vec<SearchItem> = state
        .get_smart_db()
        .fetch_all(&view)
        .await
        .map_err(|error| {
            log_db_error("global_search", &error);
            SearchError::DatabaseError
        })?;
    Ok(SearchResultView { items })
}

#[utoipa::path(
    get,
    path = "",
    summary = "Search the projects and tasks (global search)",
    description = "Part of the global search of the shared header: every business API answers \
                   `GET /api/v1/search` with the same shape, and each BFF merges them. Project \
                   returns the **projects** (`kind = project`, title or description containing \
                   `q`) and the **tasks** (`kind = task`, title containing `q`, `parent_id` = \
                   their project), case-insensitive, among the projects the caller sees (same \
                   rules as `GET /api/v1/projects/`). Titles starting with `q` come first, then \
                   the most recent.\n\n\
                   Not cached: the result depends on the term.",
    params(SearchQuery),
    responses(
        (
            status = 200,
            description = "At most `limit` projects and tasks, best match first. Empty when nothing matches.",
            body = SearchResultView,
            example = json!({
                "items": [
                    {
                        "kind": "project",
                        "id": 42,
                        "title": "Jules Ferry school renovation",
                        "excerpt": "New windows and attic insulation",
                        "parent_id": null,
                        "updated_at": "2026-11-20T09:30:00Z"
                    },
                    {
                        "kind": "task",
                        "id": 310,
                        "title": "Renovation: pick the carpenter",
                        "excerpt": null,
                        "parent_id": 42,
                        "updated_at": "2026-11-18T14:05:00Z"
                    }
                ]
            })
        ),
        (
            status = 400,
            description = "`q` is missing, shorter than 2 or longer than 100 characters once trimmed, or holds a control character; or `limit` is outside 1–20.",
            body = String,
            content_type = "text/plain",
            example = json!("Invalid `q`: must be at least 2 characters")
        ),
        (
            status = 401,
            description = "Missing `Authorization` header, invalid or expired JWT, or revoked session.",
            body = String,
            content_type = "text/plain",
            example = json!("Unauthorized: No JWT token provided.")
        ),
        (
            status = 500,
            description = "Database error while searching.",
            body = String,
            content_type = "text/plain",
            example = json!("An error occurred while accessing the database.")
        )
    ),
    tag = "Search",
    security(
        ("jwt" = [])
    )
)]
#[get("")]
pub async fn global_search(
    caller: AuthenticatedUser,
    state: web::Data<AppState>,
    query: ValidatedQuery<SearchQuery>,
) -> Result<impl Responder, SearchError> {
    let result = trigger_global_search(state, caller.id, query.into_inner()).await?;
    Ok(HttpResponse::Ok().json(result))
}
