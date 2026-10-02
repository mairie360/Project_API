use std::time::Duration;

use actix_web::{middleware, web, App, HttpServer};

use project_api::database::pg_url::build_pg_url;
use project_api::endpoints::swagger::{configure_docs, docs_enabled};
use project_api::endpoints::{config, health};

use mairie360_api_lib::env_manager::get_critical_env_var;
use mairie360_api_lib::security::JwtMiddleware;
use mairie360_api_lib::state::AppState;

use tracing_subscriber::EnvFilter;

//                                        -- MAIN FUNCTION --

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // `RUST_LOG` overrides the level; database errors are logged at `error`.
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    let redis_url = get_critical_env_var("REDIS_URL");
    let db_user = get_critical_env_var("DB_USER");
    let db_password = get_critical_env_var("DB_PASSWORD");
    let db_host = get_critical_env_var("DB_HOST");
    let db_port = get_critical_env_var("DB_PORT");
    let db_name = get_critical_env_var("DB_NAME");
    let pg_url = build_pg_url(&db_user, &db_password, &db_host, &db_port, &db_name);
    let state = AppState::new(redis_url, pg_url).await;
    wait_for_database(&state).await?;
    let data = web::Data::new(state);
    let host = get_critical_env_var("HOST");
    let port = get_critical_env_var("PORT");
    let bind_address = format!("{}:{}", host, port);

    let docs = docs_enabled();
    if docs {
        tracing::info!("API documentation served on /swagger-ui/ and /api-docs/openapi.json");
    }

    let server = HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(middleware::Logger::default())
            // Every response is JSON or plain text: forbid browsers from sniffing it as HTML.
            .wrap(middleware::DefaultHeaders::new().add(("X-Content-Type-Options", "nosniff")))
            // Swagger UI and the OpenAPI contract, only when API_DOCS_ENABLED is set.
            .configure(|cfg| configure_docs(cfg, docs))
            // Public probes
            .service(health::health)
            .service(health::ready)
            // Everything else requires a JWT
            .service(web::scope("/api").wrap(JwtMiddleware).configure(config))
    })
    .bind(bind_address)?;

    let addr = server.addrs().first().copied();
    tokio::spawn(async move {
        if let Some(addr) = addr {
            tracing::info!("Server listening on http://{addr}");
        }
    });

    server.run().await
}

/// How long the API waits for Postgres at startup before giving up.
const STARTUP_DATABASE_WAIT: Duration = Duration::from_secs(30);

/// Refuses to start without Postgres (MAIR-423): the lib builds the state even when the database
/// is unreachable, and the API would then answer `500` on every route. Retries for
/// [`STARTUP_DATABASE_WAIT`] to absorb a database that starts at the same time, then exits with an
/// error so the orchestrator restarts the pod. Redis is only checked by `GET /ready`.
async fn wait_for_database(state: &AppState) -> std::io::Result<()> {
    let deadline = tokio::time::Instant::now() + STARTUP_DATABASE_WAIT;
    loop {
        let readiness = health::check_dependencies(state, Duration::from_secs(2)).await;
        if readiness.postgres {
            return Ok(());
        }
        if tokio::time::Instant::now() >= deadline {
            tracing::error!("Postgres is unreachable, refusing to start");
            return Err(std::io::Error::other("Postgres is unreachable"));
        }
        tracing::warn!("waiting for Postgres...");
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}
