# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

`project_api` is one microservice in the **mairie360** suite: an Actix-web REST API (port `3001`) that manages projects, their tasks (and task fields) and their user membership. It was scaffolded from a generic "Rust API template" (see `README.md`), so config files still carry `#change api name` / `#change port` placeholders.

Shared infrastructure (DB, Redis, cache-aside, JWT auth, env helpers, test containers) lives in the external crate **`mairie360_api_lib`** (pinned to `1.2.2`) — read its source under `~/.cargo/registry/src/*/mairie360_api_lib-<version>/src/` when an import is unclear. `project_api` has **no direct `sqlx` dependency**; all SQL goes through the library.

## Data access (`mairie360_api_lib` 1.2.2)

`AppState` (`mairie360_api_lib::state::AppState`, built once in `main.rs` via `AppState::new(redis_url, pg_url)`) owns a `SmartDatabase` (Postgres + Redis cache-aside) reachable from handlers with `state.get_smart_db()`.

Every DB operation is described by a **query view** — a struct in `src/database/<resource>/<op>/view.rs` that stores a `Vec<QueryParam>` and implements `mairie360_api_lib::database::db_interface::ApiRequestDto` (`query_sql() -> &'static str`, `query_params() -> &[QueryParam]`, optional `cache_key`/`cache_ttl`). There is no per-operation `query.rs` anymore; `SmartDatabase` runs the SQL:

- `execute(view)` — INSERT/UPDATE/DELETE, returns `Result<(), ApiLibError>` (no row count, so "not found" is not detectable this way).
- `fetch_scalar::<T, _>(&view)` — one primitive column (e.g. `RETURNING id` → `i32`).
- `fetch_one::<T, _>(&view)` / `fetch_all::<T, _>(&view)` — `T: Serialize + DeserializeOwned`; **the SQL must return a single JSON column**, so multi-column reads wrap rows in `to_jsonb(t)`.

`QueryParam` only has `I32 / I64 / Bool / Text / Uuid / DateTime / IpAddr / OptionI32` — nullable text/enum/timestamp params are passed as `Text` and reconciled in SQL with `NULLIF($n,'')` + a `::type` cast; JSONB is passed as a serialized `Text` with `$n::jsonb`.

## Commands

Cargo aliases are defined in `.cargo/config.toml`:

| Task | Command |
|------|---------|
| Format check (CI lint) | `cargo lint_check` (`fmt --all -- --check`) |
| Format | `cargo lint_fix` (`fmt --all`) |
| Clippy (warnings = errors) | `cargo check_code` (`clippy --all-targets --all-features -- -D warnings`) |
| Regenerate `openapi.json` | `cargo open_api > openapi.json` (`run --example generate_openapi`, prints JSON to stdout) |
| Regenerate TS client | `npx orval` (reads `openapi.json` via `orval.config.js` → `generated/`) |
| Build release image | `docker build .` (distroless `Dockerfile`) |

`openapi.json`, `generated/`, `node_modules/` are git-ignored build outputs.

### Running locally

The binary needs these env vars (see `docker-compose.yml` `x-common-env`): `HOST`, `PORT`, `REDIS_URL`, `DB_USER`, `DB_PASSWORD`, `DB_HOST`, `DB_PORT`, `DB_NAME`, `JWT_SECRET`, `JWT_TIMEOUT`. The Postgres URL is assembled by `database::pg_url::build_pg_url`, which percent-encodes user, password and database name, so `DB_PASSWORD` may contain any character. Normal workflow is Docker:

```bash
docker compose up            # full stack: postgres + liquibase migrations + redis + seeder + api + nginx
docker compose watch         # same, with hot reload (cargo-watch syncs src/, Cargo.toml, Cargo.lock)
```

The DB schema is **not in this repo** — it is applied by the `ghcr.io/mairie360/liquibase-migrations` image; `init-test.sql` only seeds a couple of extra test users.

### Tests

```bash
cargo test                                   # all
cargo test test_create_project_success       # single test by name
cargo test --test integration_test queries::project::create   # one test module
```

End-to-end tests (what CI runs on `main` after the dev release, needs Docker + GHCR pull access):

```bash
./integration_test.sh    # docker-compose-integration.yml: full stack + newman replaying tests/postman/collection.json
./security_test.sh       # docker-compose-security.yml: full stack + ZAP scan of /api-docs/openapi.json
./performance_test.sh    # docker-compose-performance.yml: full stack + k6 (load-test.js)
```

The service under test in these three stacks is `image: ${IMAGE_REF}` (no `build:` block). CI sets `IMAGE_REF` to the
published `ghcr.io/mairie360/project-api:dev-<sha>` image; when it is empty the scripts build `project-api:local` from
`development.Dockerfile` first. That image is distroless (no shell, no curl), so readiness is a `project-ready` sidecar
polling `/health`, and dependent services wait for it with `service_completed_successfully`.

The ZAP scan is authenticated: `security-scan` injects a static admin JWT (`sub=1`, signed with
`JWT_SECRET=b"secret"`, see the comment in `docker-compose-security.yml`) on every request, waits for the `seeder`
service (`init-test.sql`: plain `User` accounts 2 and 3, user 1 is the Admin created by liquibase) and fails on any
alert not set to `IGNORE` / `OUTOFSCOPE` in `.zap/rules.tsv` (no `-I`). `-O http://project:3001` is required: the
spec's `servers` are unreachable from the ZAP container. Keep `rules.tsv` identical in every API. The scan fuzzes
every field, so a `500` (value too long, NUL byte, unmapped constraint violation) or a `<script>` echoed back fails
the job: validate inputs, don't silence the alert.

Both the ZAP and k6 stacks carry the OpenAPI coverage gate (MAIR-194) from mairie360/CICD `tests/`, available as
`cicd-repo/` (checked out by CI, cloned by the scripts at the pinned `cicd_version` otherwise, override with
`CICD_VERSION`; gitignored). ZAP runs with `--hook zap_hooks.py` and fails when an operation of the served spec was
never reached, or when an operation declaring `security(("jwt" = []))` only got 401/403. `load-test.js` is built on
`coverage.js` and covers every operation (MAIR-195): GET handlers run in the `reads` scenario (20 VUs) against a
project created in `setup()`, the other methods in the `writes` scenario (2 VUs), each handler creating and deleting
its own project/task so they are order-independent; one `p(95)` threshold per `op` tag (200 ms reads, 500 ms
writes) and `http_req_failed < 1%`. The spec k6 reads is the one served by the image under test, saved into the
`openapi-spec` volume by `project-ready`. **Adding an endpoint = adding its handler in `load-test.js`** (k6 aborts
at init otherwise), nothing to do for ZAP. `init-test.sql` also seeds the rows of the spec's path examples (project
12, task 87, user 42) so ZAP reaches real rows, until its own `DELETE` removes them.

Request bodies with text fields are extracted with `endpoints::validation::ValidatedJson` instead of `web::Json`:
the view implements `Validate` (length matching the Postgres column, no control character, no `<` / `>` in names,
descriptions and labels) and an invalid value answers `400` naming the field before the handler runs. Document the
rules in the view's `#[schema]` and the handler's `400` response. Map constraint violations of the lib's `DbError`
(`ForeignKeyViolation`, `UniqueViolation`) to `4xx` instead of `500`.

`tests/postman/collection.json` is a Postman v2.1 collection (importable in the app) and
`tests/postman/environment.json` its variables; the compose file overrides `baseUrl` with `--env-var` so the
committed default (`http://localhost:3001`) stays usable from a host shell. There is no login route here, so the
collection pre-request script forges the HS256 JWTs itself (claims `sub`/`role`/`exp`, signed with the stack's
`JWT_SECRET`) for the seeded Admin (user 1) and a plain agent (user 2, from `init-test.sql`). The scenario creates
its own project and deletes it at the end, so it is replayable against a persistent database.

`tests/routing_test.rs` needs no Docker: it mounts `endpoints::config` under `/api` in an actix test app and checks that every `/api/v1` operation published by `ApiDoc` (the contract `@mairie360/project-api-openapi` is generated from) hits a real route and has no empty segment. It catches a `scope(...)` that drifts from the `doc.rs` nesting or a `#[utoipa::path]` without the right `path` (utoipa appends it to the nest path, so `#[delete("/")]` needs `path = ""`, `#[patch("/close")]` needs `path = "close"`).

Integration tests (`tests/queries/`) require a **running Docker daemon and network access to ghcr.io**: each `#[tokio::test]` calls `mairie360_api_lib::test_setup::queries_setup::get_shared_db()`, which starts a `ghcr.io/mairie360/database` Postgres container (published on a random host port), runs Liquibase migrations against it, truncates + seeds it once per test run, and hands back a connection string. `tests/common::get_smart_db(url)` wraps it in a `SmartDatabase` (real Redis not needed — no query view sets a `cache_key`). Tests drive the query views through `execute` / `fetch_*` and hit real SQL — there is no compile-time query checking.

## Architecture

### Request pipeline (`src/main.rs`)

`HttpServer` mounts, in order: Swagger UI (`/swagger-ui/*`, `/api-docs/openapi.json`), public `health::health` (`/health`) and `hello::hello` (`/`), then `web::scope("/api").wrap(JwtMiddleware).configure(endpoints::config)`. Everything under `/api` requires a valid JWT; handlers get the caller via the `AuthenticatedUser { id }` extractor from `mairie360_api_lib::security`.

Route tree: `endpoints::config` → `v1::config` → `/v1/projects` → `projects::{get,post}` + `project_id::config` (`/{project_id}/close`, `/delete`, `/tasks/...`, `/users/...`) + `templates::config`. `src/lib.rs` re-exports `database` and `endpoints`; the crate is both a lib and a bin so `examples/` and `tests/` can depend on the lib.

### Endpoint module convention (`src/endpoints/v1/…`)

The module tree **mirrors the URL path**. Each leaf HTTP method is its own directory containing:

- **`endpoint.rs`** — the handler (`#[utoipa::path(...)]` + actix `#[get]`/`#[post]`/… macro), a local `XxxError` enum implementing `Display` + actix `ResponseError` (maps variants to status codes), and an inner `async fn trigger_xxx(state, user_id, view) -> Result<_, XxxError>` that builds a query view and calls `state.get_smart_db().execute/fetch_*`, mapping errors to `XxxError::DatabaseError`.
- **`view.rs`** — request/response DTOs deriving `serde` + `utoipa::ToSchema`; request views implement `TryFrom<web::Json<Self>>` as the validation hook.
- **`mod.rs`** — declares submodules; a parent `mod.rs` exposes `pub fn config(cfg: &mut web::ServiceConfig)` that composes `web::scope(...)` and registers services.
- **`doc.rs`** — a `utoipa::OpenApi` struct, nested via `#[openapi(nest(...))]` up to `src/endpoints/swagger.rs::ApiDoc`. When you add an endpoint you must also register its `__path_*` and schemas in the relevant `doc.rs`.

### Database layer (`src/database/<resource>/<operation>/view.rs`)

Resources: `project`, `tasks` (+ `tasks/fields`), `users` (project membership). Each operation is a single `view.rs` holding the `XxxQueryView` (`ApiRequestDto` impl, see "Data access" above) and, for reads, the result DTO it deserialises into (`ProjectView`, `Task`, …) deriving `serde::{Serialize, Deserialize}`. Endpoint HTTP DTOs are separate types; the `trigger_*` fn converts between them.

### Visibility and task collaboration

`GET /projects/` and `GET /projects/{project_id}/` only return projects visible to the caller, using the
`project_visible_to_user_sql!` fragment (`src/database/project/mod.rs`): Admin/Maire see everything, a Responsable
also sees projects owned by or shared with a member of one of their groups, everyone else sees projects they own
or are a member of. A project that is not visible answers 404. Every operation under `/projects/{project_id}` calls
`endpoints/v1/projects/access.rs::require_access` (one `ProjectAccessQueryView` query): reads need visibility;
writes on the project, its tasks and members need visibility plus the Admin/Maire/Responsable role (403
otherwise); task collaboration and `PATCH` on a task are also open to the task's assignee, who may only change
the status. Creating a project requires one of those roles. History entries are limited to `task_created`,
`task_updated` and `status_changed`, always signed by the caller. Task comments and free history live in `tasks.custom_fields`
(`comments`, `history`, next to the ordered `fields` list written at creation); status changes come from
`task_history` (DB trigger).

### Deployment

`Dockerfile` = multi-stage release build onto `gcr.io/distroless/cc-debian12`. `development.Dockerfile` + `entrypoint.sh` = `cargo watch` dev container used by compose. `nginx.conf` reverse-proxies `:80` → api `:3001`. CI (`.github/workflows/cicd.yml`) just calls the reusable `mairie360/CICD` workflow, which builds/pushes the `project-api` image and runs the three `*_test.sh` scripts with `IMAGE_REF` set to the `dev-<sha>` image published by `release-dev` (newman, no Postman account involved).

## Pull request reviewers

Every PR requests a review from the whole team, minus its author: `CarolinHugo`, `LAURETbenjamin`, `MathTek` and `Quentintnrl` (`gh pr create … --reviewer CarolinHugo,LAURETbenjamin,MathTek`). `.github/CODEOWNERS` makes GitHub request them automatically as well.
