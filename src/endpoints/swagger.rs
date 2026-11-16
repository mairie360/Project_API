use crate::endpoints::health::HealthDoc;
use crate::endpoints::v1::doc::V1Doc;
use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};
use utoipa::{Modify, OpenApi};

// Root ApiDoc
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Project API — Mairie 360",
        version = "1.0.0",
        description = "\
Project management API of the **Mairie 360** platform: projects, tasks, custom fields, \
comments, history and members. It holds neither the accounts nor the roles, which live in \
Core API.

## Authentication

Every route under `/api` is protected by `JwtMiddleware`. The token is obtained from Core API \
(`POST /api/v1/auth/login`) and sent here in the `Authorization` header (`Bearer <jwt>`).

## Access rights

Every operation on a project checks the caller's rights before acting, which gives the \
following grid:

| Requirement | Who passes | Operations |
| --- | --- | --- |
| View the project | any member of the project | reads of the project, its tasks and members |
| Manage the project | manager of the project | creation, update, closing, deletion, management of the tasks and members |
| Act on the task | manager of the project, or agent assigned to the task | collaboration, comments, history, status change |

Creating a project is reserved to the Admin, Maire and Responsable roles.

Project templates (`/api/v1/project-templates/`, MAIR-566) are read by any signed-in user and \
managed by the Admin and Responsable roles.

A project the caller cannot see is treated as **non-existent**: the answer is `404`, not \
`403`, so that its existence is not revealed. `403` only shows up on a project the caller can \
see but not change.

## Error format

Error responses (`4xx` and `5xx`) have a **`text/plain`** body holding the error message, not a \
JSON object. Every response carries `X-Content-Type-Options: nosniff`.

Statuses returned across the API, before the handler runs:

| Status | Meaning |
| --- | --- |
| `400` | URL segment that is not an integer, malformed JSON body, or a field breaking its \
validation rules (length, control characters); the body names the first invalid field, e.g. \
``Invalid `name`: must not contain control characters``. `<` and `>` are ordinary text. |
| `401` | `Authorization` header missing or malformed, invalid or expired JWT, or revoked session. |
| `500` | Database or Redis failure. |
",
        contact(
            name = "Mairie 360 team",
            url = "https://github.com/mairie360"
        ),
        license(
            name = "Proprietary",
            identifier = "LicenseRef-mairie360-proprietary"
        )
    ),
    servers(
        (url = "http://localhost:3001", description = "Local development (cargo run)"),
        (url = "http://development.mairie360.fr", description = "Development Docker stack (nginx)")
    ),
    tags(
        (name = "Projects", description = "Life cycle of the projects: creation, reading, update, closing and deletion."),
        (name = "Tasks", description = "Tasks of a project, their custom fields, comments and history, and the tasks visible to the caller across every project (`GET /api/v1/tasks/`)."),
        (name = "Users", description = "Members of a project: listing, adding and removing."),
        (name = "Statuses", description = "Task statuses of a project (MAIR-566): its Kanban columns, each in one of the categories `Todo`, `InProgress`, `Completed`."),
        (name = "Templates", description = "Project templates (MAIR-566): the statuses and custom fields copied into a project created from one. Managed by Admin and Responsable."),
        (name = "probes", description = "Unauthenticated liveness (`/health`) and readiness (`/ready`) probes, used by Docker and Kubernetes.")
    ),
    nest(
        (path = "/api/v1", api = V1Doc),
        (path = "/", api = HealthDoc),
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

/// Without this modifier, the 17 operations declaring `security(("jwt" = []))` would reference a
/// scheme missing from the contract: Swagger UI would offer no "Authorize" button and the
/// generated clients would point to nothing.
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "jwt",
            SecurityScheme::Http(
                Http::builder()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .description(Some("JWT issued by Core API (`POST /api/v1/auth/login`)."))
                    .build(),
            ),
        )
    }
}

/// Set to `true` to serve Swagger UI (`/swagger-ui/`) and the contract (`/api-docs/openapi.json`).
/// Off by default, so off in production (MAIR-424); the dev and test stacks enable it, ZAP and k6
/// read the spec from there.
pub const API_DOCS_ENABLED: &str = "API_DOCS_ENABLED";

/// Whether the value of [`API_DOCS_ENABLED`] enables the docs: only `true` (case-insensitive) does,
/// an absent or any other value keeps them off.
#[must_use]
pub fn api_docs_enabled(value: Option<&str>) -> bool {
    value.is_some_and(|v| v.trim().eq_ignore_ascii_case("true"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_docs_are_off_unless_explicitly_enabled() {
        assert!(api_docs_enabled(Some("true")));
        assert!(api_docs_enabled(Some(" TRUE ")));
        assert!(!api_docs_enabled(None));
        assert!(!api_docs_enabled(Some("")));
        assert!(!api_docs_enabled(Some("1")));
        assert!(!api_docs_enabled(Some("false")));
    }
}
