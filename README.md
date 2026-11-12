# Project API

Projects, their tasks (statuses, comments, history, archiving), task statuses per project, project templates and project membership. Consumed by `BFF_Project` and `BFF_Dashboard`.

- Crate `project_api`, image `ghcr.io/mairie360/project-api`, port **3001** (Actix-web, PostgreSQL, Redis).
- Shared infrastructure (database access and cache, JWT middleware, env helpers, test containers) comes from [`mairie360_api_lib`](https://crates.io/crates/mairie360_api_lib).
- Generated from `API_template`; the database schema is not in this repo (it lives in `Devops/Database`, applied by the `liquibase-migrations` image).

## API documentation

There is no hand-written endpoint list: the OpenAPI document is generated from the code (`utoipa`) and is the reference.

- Swagger UI at `/swagger-ui/` and the spec at `/api-docs/openapi.json`, served only when `API_DOCS_ENABLED=true` (every compose stack sets it, production does not).
- Generate the spec offline: `cargo open_api > openapi.json` (`cargo run --example generate_openapi`).
- Published TypeScript client and contract: [`@mairie360/project-api-openapi`](https://github.com/orgs/mairie360/packages) on GitHub Packages (`npx orval` regenerates a client from `openapi.json` into `generated/`, see `orval.config.js`).
- Probes, outside `/api`: `GET /health` (liveness) and `GET /ready` (readiness: Postgres and Redis).
- Every route under `/api` requires `Authorization: Bearer <session JWT>`.

No rate limiting in this API (the ingress / BFF layer does it).

## Environment variables

Required (the process panics when one is missing):

| Variable | Role |
| --- | --- |
| `REDIS_URL` | Redis URL (cache and revocation list). Redis stays optional at runtime: the cache degrades to Postgres. |
| `DB_USER` / `DB_PASSWORD` | Postgres role of this API (`project_api`) and its password. |
| `DB_HOST` / `DB_PORT` / `DB_NAME` | Postgres address and database. |
| `HOST` / `PORT` | Listen address and port (`3001` in the dev stack). |
| `JWT_PUBLIC_KEY` | Core_API's Ed25519 public key (PEM, several `PUBLIC KEY` blocks during a rotation): only Core signs session JWTs (MAIR-680). No `JWT_SECRET`; the dev stack sets `JWT_ALLOW_WEAK_SECRET=true` with the lib's test key, never in a deployment. |
| `JWT_TIMEOUT` | Session lifetime in seconds, read by `mairie360_api_lib` (positive integer). |

Optional:

| Variable | Role |
| --- | --- |
| `API_DOCS_ENABLED` | `true` serves Swagger UI and `/api-docs/openapi.json`; leave it unset in production. |
| `DB_CONNECT_TIMEOUT` | Seconds to wait for Postgres at startup before panicking (lib, default 30). |
| `DB_POOL_MIN_CONNECTIONS` / `DB_POOL_MAX_CONNECTIONS` / `DB_POOL_IDLE_TIMEOUT_SECS` | Postgres pool sizing (lib, defaults 1 / 10 / 600). |
| `RUST_LOG` / `LOG_FORMAT` | Log level (default `info`); `LOG_FORMAT=json` for one JSON object per line. |
| `OTEL_EXPORTER_OTLP_ENDPOINT` / `OTEL_SERVICE_NAME` / `OTEL_SDK_DISABLED` | OTLP/HTTP trace export, off unless an endpoint is set. |

`docker-compose.yml` (`x-common-env`) shows working values for the dev stack.

## Run locally

Docker and Docker Compose are enough (pulling the `ghcr.io/mairie360/*` images needs registry access):

```bash
docker compose up --watch   # Postgres + Liquibase + seeder + Redis + API (hot reload, src/ synced) + nginx
```

The API listens on `localhost:3001`. The dev stack signs nothing: to call `/api`, sign a token with the lib's test key (see `CLAUDE.md`, "Session tokens").

Without Docker for the API itself, export the variables above (Postgres and Redis must be reachable) and run `cargo run`.

## Tests and checks

```bash
cargo lint_check    # cargo fmt --all -- --check
cargo check_code    # clippy --all-targets --all-features -- -D warnings
cargo test          # needs Docker: Postgres/Redis testcontainers, ghcr.io/mairie360 access
cargo cov_test      # llvm-cov with a 60 % line gate (only main.rs and lib.rs excluded)
./integration_test.sh   # newman replays tests/postman/collection.json (see below)
./security_test.sh      # OWASP ZAP scan of the served OpenAPI spec
./performance_test.sh   # k6 load test (load-test.js)
```

The three `*_test.sh` stacks run in CI on `main` against the published `dev-<sha>` image (`IMAGE_REF`); locally they build `project-api:local` from `development.Dockerfile` when it is empty.

## Integration tests (newman)

The end-to-end scenario lives in `tests/postman/collection.json` (Postman v2.1 collection) with
its variables in `tests/postman/environment.json`. CI replays it with the `postman/newman` image
against the published `dev-<sha>` API image (`IMAGE_REF`), without any Postman account. Locally,
leave `IMAGE_REF` empty and the script builds `project-api:local` from `development.Dockerfile`:

```bash
./integration_test.sh
```

The script starts `docker-compose-integration.yml` (Postgres + Liquibase + seeder + Redis + API +
newman), waits for the `newman` service, prints its report and exits with its status (`--bail`
stops at the first failing request). To iterate on the collection against a stack already running
on `localhost:3001`:

```bash
docker run --rm --network host -v "$PWD/tests/postman:/etc/newman:ro" postman/newman:6.1.3-alpine \
  run collection.json --environment environment.json
```

## Session tokens

Only Core_API signs session JWTs (MAIR-680, Ed25519 / `EdDSA`). This API needs `JWT_PUBLIC_KEY`, Core's public
key as PEM (several `PUBLIC KEY` blocks during a rotation, `\n`-escaped line breaks accepted), and no
`JWT_SECRET`: while `JWT_SECRET` is set, HS256 tokens signed with it are still accepted (transition), so remove it
once `JWT_PUBLIC_KEY` is deployed. The dev compose stack uses the public test key of `mairie360_api_lib` with
`JWT_ALLOW_WEAK_SECRET=true`; never set that variable in a deployment. The test scripts generate a fresh key pair
per run (`test_secrets.sh`).
