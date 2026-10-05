use std::time::Duration;

use actix_web::{get, web, HttpResponse, Responder};
use mairie360_api_lib::state::AppState;
use serde::Serialize;
use utoipa::{OpenApi, ToSchema};

use crate::database::service::ping::PingQueryView;

/// Redis key read by the probe (`EXISTS`, under the API's key prefix): it never exists.
const REDIS_PROBE_KEY: &str = "readiness-probe";

/// Handles a GET request to the /health endpoint: liveness only.
#[utoipa::path(
    get,
    path = "health",
    summary = "Liveness probe",
    description = "Answers `OK` as soon as the process accepts connections. Not authenticated. \
                   It checks neither Postgres nor Redis, on purpose: a failing dependency must \
                   take the pod out of the service (`GET /ready`), not restart it. Use it as the \
                   Kubernetes `livenessProbe`.",
    responses(
        (
            status = 200,
            description = "The process accepts connections.",
            body = String,
            content_type = "text/plain",
            example = json!("OK")
        )
    ),
    tag = "Service"
)]
#[get("/health")]
pub async fn health() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

/// State of each dependency, as returned by `GET /ready`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
pub struct Readiness {
    /// `true` when `SELECT 1` succeeded on Postgres.
    #[schema(example = true)]
    pub postgres: bool,
    /// `true` when Redis answered a read (`EXISTS`).
    #[schema(example = true)]
    pub redis: bool,
}

impl Readiness {
    pub fn is_ready(&self) -> bool {
        self.postgres && self.redis
    }
}

/// Queries Postgres and Redis; each check gives up after `timeout`.
pub async fn check_dependencies(state: &AppState, timeout: Duration) -> Readiness {
    let postgres = async {
        matches!(
            tokio::time::timeout(
                timeout,
                state
                    .get_smart_db()
                    .fetch_scalar::<i32, _>(&PingQueryView::default())
            )
            .await,
            Ok(Ok(1))
        )
    };
    let redis = async {
        matches!(
            tokio::time::timeout(timeout, state.get_redis().key_exist(REDIS_PROBE_KEY)).await,
            Ok(Ok(_))
        )
    };
    let (postgres, redis) = tokio::join!(postgres, redis);
    Readiness { postgres, redis }
}

/// Handles a GET request to the /ready endpoint: readiness, with Postgres and Redis.
#[utoipa::path(
    get,
    path = "ready",
    summary = "Readiness probe",
    description = "Queries Postgres (`SELECT 1`) and Redis (a read), each with a 2-second \
                   timeout, and answers `200` only when both respond. Not authenticated. Use it \
                   as the Kubernetes `readinessProbe` (and `startupProbe`): a pod whose database \
                   is unreachable is taken out of the service instead of answering `500`. Redis \
                   is required because the revocation list of sessions lives there.",
    responses(
        (
            status = 200,
            description = "Both dependencies answered.",
            body = Readiness,
            example = json!({ "postgres": true, "redis": true })
        ),
        (
            status = 503,
            description = "Postgres or Redis did not answer in time; the body tells which one.",
            body = Readiness,
            example = json!({ "postgres": false, "redis": true })
        )
    ),
    tag = "Service"
)]
#[get("/ready")]
pub async fn ready(state: web::Data<AppState>) -> impl Responder {
    let readiness = check_dependencies(&state, Duration::from_secs(2)).await;
    if readiness.is_ready() {
        HttpResponse::Ok().json(readiness)
    } else {
        tracing::warn!(?readiness, "readiness probe failed");
        HttpResponse::ServiceUnavailable().json(readiness)
    }
}

#[derive(OpenApi)]
#[openapi(paths(health, ready), components(schemas(Readiness)))]
pub struct HealthDoc;
