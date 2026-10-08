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

Ids are `u64` in the API and `INT4` in the database: convert them with the lib's `id_to_sql` / `id_from_sql`
(saturating, so `2^32 + 1` no longer wraps to row `1`), never with `as`; `src/lib.rs` denies the
`cast_possible_truncation` / `cast_possible_wrap` / `cast_sign_loss` lints (MAIR-422).

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

The binary needs these env vars (see `docker-compose.yml` `x-common-env`): `HOST`, `PORT`, `REDIS_URL`, `DB_USER`, `DB_PASSWORD`, `DB_HOST`, `DB_PORT`, `DB_NAME`, `JWT_SECRET`, `JWT_TIMEOUT`, plus the optional `API_DOCS_ENABLED` (only `true` serves Swagger UI and `/api-docs/openapi.json`; off by default so off in production, on in every compose stack, MAIR-424). The Postgres URL is assembled by `database::pg_url::build_pg_url`, which percent-encodes user, password and database name, so `DB_PASSWORD` may contain any character. Normal workflow is Docker:

```bash
docker compose up            # full stack: postgres + liquibase migrations + redis + seeder + api + nginx
docker compose watch         # same, with hot reload (cargo-watch syncs src/, Cargo.toml, Cargo.lock)
```

`mairie360_api_lib` (2.0+) refuses to start with a missing, short (< 32 bytes) or well-known `JWT_SECRET`
(MAIR-428). The four compose stacks keep the public test value `b"secret"` (the static ZAP / k6 admin token and the
Postman script sign with it) and set `JWT_ALLOW_WEAK_SECRET: "true"` next to it: without it the API panics at
startup and every stack fails. A deployment never sets that variable and gets its own random secret, distinct per
instance.

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
polling `/ready`, and dependent services wait for it with `service_completed_successfully`.

The ZAP scan is authenticated: `security-scan` injects a static admin JWT (`sub=1`, signed with
`JWT_SECRET=b"secret"`, see the comment in `docker-compose-security.yml`) on every request, waits for the `seeder`
service (`init-test.sql`: plain `User` accounts 2 and 3, user 1 is the Admin created by liquibase) and fails on any
alert not set to `IGNORE` / `OUTOFSCOPE` in `.zap/rules.tsv` (no `-I`). `-O http://project:3001` is required: the
spec's `servers` are unreachable from the ZAP container. Keep `rules.tsv` identical in every API. The scan fuzzes
every field, so a `500` (value too long, NUL byte, unmapped constraint violation) fails the job: validate inputs,
don't silence the alert. The XSS rules (40012, 40014, 40016, 40017) are the exception, set to `IGNORE` (MAIR-426):
`<` and `>` are legitimate text, and echoing them in a JSON or `text/plain` body served with `nosniff` is not an
injection; escaping is the fronts' job. Never answer HTML.

Both the ZAP and k6 stacks carry the OpenAPI coverage gate (MAIR-194) from mairie360/CICD `tests/`, available as
`cicd-repo/` (checked out by CI, cloned by the scripts at the pinned `cicd_version` otherwise, override with
`CICD_VERSION`; gitignored). ZAP runs with `--hook zap_hooks.py` and fails when an operation of the served spec was
never reached, or when an operation declaring `security(("jwt" = []))` only got 401/403. `load-test.js` is built on
`coverage.js` and covers every operation (MAIR-195), under a high load on a volume seed (MAIR-474): the performance
stack's `seeder` also runs `init-perf.sql` (5 000 projects, 50 000 tasks, 2 100 accounts in 100 teams, hot project 12
with 2 000 tasks, task 87 with 1 000 comments and history entries). GET handlers run in the `reads` scenario (up to
100 VUs) as the Admin, a seeded Responsable or a seeded agent (tokens signed in k6 with the stack's `JWT_SECRET`) on
random pages; the other methods in the `writes` scenario (10 VUs), each handler creating and deleting its own
project/task so they are order-independent (the task writes share one project, so they also queue on its lock); a
`list_rush` scenario sends `GET /projects/` at a fixed 100 req/s. Thresholds: one `p(95)` per `op` tag (200 ms
reads, 500 ms writes), `checks == 100%` (status and seeded rows), `dropped_iterations == 0`, `http_req_failed == 0`. Two load profiles (`K6_PROFILE`, passed by the compose file): `ci` (default) is what the 4 vCPU CI runner holds with the strict thresholds (30 readers, 4 writers, rush at 30 req/s); `stress` is the high load (100 readers, 10 writers, 100 req/s), run by hand with `K6_PROFILE=stress ./performance_test.sh` to find the breaking point, not on every push. Keep `init-perf.sql` and
the id ranges at the top of `load-test.js` in step. The spec k6 reads is the one served by the image under test,
saved into the `openapi-spec` volume by `project-ready`. **Adding an endpoint = adding its handler in
`load-test.js`** (k6 aborts
at init otherwise), nothing to do for ZAP. `init-test.sql` also seeds the rows of the spec's path examples (project
12, task 87, user 42) so ZAP reaches real rows, until its own `DELETE` removes them.

Request bodies with text fields are extracted with `endpoints::validation::ValidatedJson` instead of `web::Json`:
the view implements `Validate` (length matching the Postgres column, no control character) and an invalid value
answers `400` naming the field before the handler runs. Do **not** refuse `<` or `>`: "budget > 10 000 €" is
ordinary text (MAIR-426). Document the
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

Handler tests (`tests/endpoints/`, MAIR-419) run in the same binary and database: `init_app!` mounts the real
`/api` scope behind `JwtMiddleware`, `jwt_for(user_id)` signs tokens with a test secret set in-process, and
`access::scenario` builds a project owned by a Responsable with a plain member, an assignee and two outsiders.
`token_refusals.rs` sweeps every operation of `ApiDoc` declaring `jwt`: `401` without a token, with another
scheme, garbage, another secret, an expired token, `alg: none`, a payload swapped under a valid signature or an
asymmetric algorithm, and `404` for an unknown or archived account; a new route is covered by documenting it.
They assert the refusals (`401` without a valid JWT, `404` for a project the caller cannot see, `403` for a
member without manager role, assignee limited to the status, task reachable only through its own project) and
an end-to-end manager flow. **A new route or access rule gets its negative test here.** `cargo cov` / `cov_test`
only exclude `main.rs` and `lib.rs`: `endpoints/` counts toward the 60 % line gate.

## Architecture

### Request pipeline (`src/main.rs`)

`main` refuses to start while Postgres does not answer: `AppState::new` (lib 3.0.0) retries for `DB_CONNECT_TIMEOUT` seconds (default 30), then panics. `HttpServer` mounts, in order: Swagger UI (`/swagger-ui/*`, `/api-docs/openapi.json`) only when `API_DOCS_ENABLED=true` (`swagger::api_docs_enabled`), the public probes `health::health` (`/health`, liveness, no dependency) and `health::ready` (`/ready`, readiness: Postgres `SELECT 1` + Redis read, 2 s each; `200 ready` or `503 not ready: postgres, redis` — MAIR-423), then `web::scope("/api").wrap(JwtMiddleware).configure(endpoints::config)`. Everything under `/api` requires a valid JWT (the probes are not mounted there); handlers get the caller via the `AuthenticatedUser { id }` extractor from `mairie360_api_lib::security`.

Route tree: `endpoints::config` → `v1::config` → `/v1/projects` → `projects::{get,post}` + `project_id::config` (`/{project_id}/close`, `/delete`, `/tasks/...`, `/users/...`). `src/lib.rs` re-exports `database` and `endpoints`; the crate is both a lib and a bin so `examples/` and `tests/` can depend on the lib.

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
the status. Creating a project requires one of those roles. A task can only be assigned to the owner or a
member of its project (`assignable_to_project_sql!`, `400` otherwise), so an assignee always sees the project.

Reads check access with `require_access` (one query). **Writes** go through `access.rs::begin_write`
(MAIR-420): it opens a `SmartTransaction`, locks the project row (`LockProjectQueryView`, `FOR UPDATE`), checks
access in that transaction and hands it back; the handler runs its queries on it and ends with
`access::commit`. Writes on one project are therefore serialized, the access check cannot be invalidated before
the write, and a multi-query write is all-or-nothing (an early `?` drops the transaction, which rolls back).
Removing a member also unassigns their tasks of the project in the same transaction.

Since MAIR-393 (database `v1.8.0` schema): the task description is the `tasks.description` column, comments
are rows of `task_comments`, and the history is `task_history`, written **only** by the database trigger
`fn_log_task_change` (`task_created`, `task_updated` with `{"<field>": {"from", "to"}}`, `status_changed`).
Every task write sets `tasks.updated_by` to the caller, which signs the history; there is no route to write
history and `project_api` only has `SELECT` on that table. History labels are generated in
`database/tasks/collaboration/view.rs` (`From<TaskHistoryRow>`). `tasks.custom_fields` only holds `fields`.

List endpoints (`GET /projects/`, `GET /projects/{id}/` for its tasks, `GET …/tasks/`, `GET …/collaboration`,
`GET …/users/` — MAIR-425; the members embedded in `GET /projects/{id}/` are the first 100 + `users_total`)
take `limit` (default 100, clamped to 1–500) / `offset` (`endpoints::pagination::PageParams`) and return a
total; their query views select `paged_rows_sql!` over rows numbered `rn` and are read with
`fetch_one::<PagedRows<T>, _>`. Rate limiting is not done here (same rule as `API_template`): every call comes
from a BFF, so it belongs to the ingress / BFF layer. Database errors are logged with `endpoints::db_error::log_db_error` before
answering `500` (`tracing`, level from `RUST_LOG`, default `info`); a handler that turns some of them into a
client status maps them with `db_error::classify_db_error` (`Conflict` / `InvalidReference` / `NotFound` logged at `info`,
`Internal` at `error`) instead of matching `DbError` by hand (MAIR-421). A path segment that is not a valid id
answers `400` (`PathConfig` in `endpoints::config`), as every route documents.

### Observability (MAIR-503)

`src/telemetry.rs` (same approach as the Core API POC, MAIR-131) exports traces over OTLP/HTTP (protobuf),
opt-in and driven by the standard `OTEL_*` variables: nothing changes unless `OTEL_EXPORTER_OTLP_ENDPOINT` (or
`OTEL_EXPORTER_OTLP_TRACES_ENDPOINT`) is set, and `OTEL_SDK_DISABLED=true` forces it off. The endpoint is an
agent that relays to Scaleway Cockpit (OTel Collector or Grafana Alloy, e.g. `http://alloy:4318`, `/v1/traces` is
appended); `OTEL_SERVICE_NAME` defaults to `project-api`, `OTEL_EXPORTER_OTLP_HEADERS` carries a token when the
endpoint needs one. `telemetry::init()` always installs the stdout logs (`RUST_LOG`, default `info`); a failure to
build the exporter is printed and never stops the API.

- `main.rs` wraps the app in `tracing_actix_web::TracingLogger`, inside the request log (`middleware::Logger`,
  kept), so requests refused by `JwtMiddleware` get a span too. Root spans are named `<METHOD> <route pattern>`,
  carry `http.*` attributes and continue an incoming `traceparent`.
- No personal data leaves in a span (MAIR-290, MAIR-501): `telemetry::Redact` drops `http.client_ip` and the
  query string of `http.target` before the export, and the trace layer ignores the request log. Build providers
  with `telemetry::tracer_provider`, never `SdkTracerProvider::builder()` directly, so the redaction always
  applies. The root span also holds them in memory, so `telemetry::log_layer` hides it from the stdout logs (the
  fmt layer would print its fields in front of every event of the request).
- The SQL of `mairie360_api_lib` (sqlx) appears as span **events** (`db.statement` with `$n` placeholders, never
  the bound values, plus `elapsed` and the row counts), not as child spans.
- The `opentelemetry*`, `opentelemetry-otlp`, `opentelemetry_sdk`, `tracing-opentelemetry` and
  `tracing-actix-web` versions are coupled (`tracing-actix-web` 0.7 supports OpenTelemetry up to 0.32,
  `tracing-opentelemetry` 0.33): bump them together, never one alone.
- `tests/endpoints/telemetry.rs` asserts the span, the continued trace id and the SQL events against an
  in-memory exporter, and (without database) that neither the spans nor the logs carry the query string or the
  client address.

### Deployment

`Dockerfile` = multi-stage release build onto `gcr.io/distroless/cc-debian12`, copied from `API_template` (MAIR-427): images pinned by digest, a dependency-only layer, `cargo build --release --locked`. `.cargo/audit.toml`, `.github/workflows/cicd.yml` and `src/main.rs` are the template's too: keep them in sync with it rather than editing them here alone. `development.Dockerfile` + `entrypoint.sh` = `cargo watch` dev container used by compose. `nginx.conf` reverse-proxies `:80` → api `:3001`. CI (`.github/workflows/cicd.yml`) just calls the reusable `mairie360/CICD` workflow, which builds/pushes the `project-api` image and runs the three `*_test.sh` scripts with `IMAGE_REF` set to the `dev-<sha>` image published by `release-dev` (newman, no Postman account involved).

## Pull request reviewers

Every PR requests a review from the whole team, minus its author: `CarolinHugo`, `LAURETbenjamin`, `MathTek` and `Quentintnrl` (`gh pr create … --reviewer CarolinHugo,LAURETbenjamin,MathTek`). `.github/CODEOWNERS` makes GitHub request them automatically as well.
