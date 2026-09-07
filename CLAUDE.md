# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

`project_api` is one microservice in the **mairie360** suite: an Actix-web REST API (port `3001`) that manages projects, their tasks (and task fields) and their user membership. It was scaffolded from a generic "Rust API template" (see `README.md`), so config files still carry `#change api name` / `#change port` placeholders.

Shared infrastructure (DB, Redis, cache-aside, JWT auth, env helpers, test containers) lives in the external crate **`mairie360_api_lib`** (pinned to `1.2.0`) — read its source under `~/.cargo/registry/src/*/mairie360_api_lib-<version>/src/` when an import is unclear. `project_api` has **no direct `sqlx` dependency**; all SQL goes through the library.

## Data access (`mairie360_api_lib` 1.2.0)

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

The binary needs these env vars (see `docker-compose.yml` `x-common-env`): `HOST`, `PORT`, `REDIS_URL`, `DB_USER`, `DB_PASSWORD`, `DB_HOST`, `DB_PORT`, `DB_NAME`, `JWT_SECRET`, `JWT_TIMEOUT`. Normal workflow is Docker:

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

Integration tests (`tests/queries/`) require a **running Docker daemon and network access to ghcr.io**: each `#[tokio::test]` calls `mairie360_api_lib::test_setup::queries_setup::get_shared_db()`, which starts a `ghcr.io/mairie360/database` Postgres container (host networking, port 5432), runs Liquibase migrations against it, truncates + seeds it once per test run, and hands back a connection string. `tests/common::get_smart_db(url)` wraps it in a `SmartDatabase` (real Redis not needed — no query view sets a `cache_key`). Tests drive the query views through `execute` / `fetch_*` and hit real SQL — there is no compile-time query checking.

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

### Deployment

`Dockerfile` = multi-stage release build onto `gcr.io/distroless/cc-debian12`. `development.Dockerfile` + `entrypoint.sh` = `cargo watch` dev container used by compose. `nginx.conf` reverse-proxies `:80` → api `:3001`. CI (`.github/workflows/cicd.yml`) just calls the reusable `mairie360/CICD` workflow, which builds/pushes the `project-api` image and runs a Postman collection.
