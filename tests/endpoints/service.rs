//! Probes: `/health` is liveness only, `/ready` answers 200 only when Postgres and Redis respond.

use actix_web::test::TestRequest;
use actix_web::{web, App};
use mairie360_api_lib::state::AppState;
use mairie360_api_lib::test_setup::db_setup::start_postgres_container;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use mairie360_api_lib::test_setup::redis_setup::start_redis_container;
use project_api::endpoints::health;
use serial_test::serial;

use super::{get, status, TestContext};
use crate::init_app;

/// Nothing listens on port 1: the connection is refused immediately.
const UNREACHABLE_REDIS: &str = "redis://127.0.0.1:1";

async fn ready(state: AppState) -> (u16, String) {
    let app = actix_web::test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .service(health::health)
            .service(health::ready),
    )
    .await;
    assert_eq!(
        status(&app, TestRequest::get().uri("/health")).await,
        200,
        "liveness does not depend on the database"
    );
    let response =
        actix_web::test::call_service(&app, TestRequest::get().uri("/ready").to_request()).await;
    let code = response.status().as_u16();
    let body = actix_web::test::read_body(response).await;
    (code, String::from_utf8(body.to_vec()).unwrap())
}

#[actix_web::test]
#[serial]
async fn ready_when_postgres_and_redis_answer() {
    let (_container, pg_url) = get_shared_db().await;
    let (_redis, redis) = start_redis_container().await;
    let state = AppState::new(redis.url.clone(), pg_url.to_string()).await;

    let (code, body) = ready(state).await;
    assert_eq!(code, 200);
    assert_eq!(body, "ready");
}

#[actix_web::test]
#[serial]
async fn not_ready_without_redis() {
    let (_container, pg_url) = get_shared_db().await;
    let state = AppState::new(UNREACHABLE_REDIS.to_string(), pg_url.to_string()).await;

    let (code, body) = ready(state).await;
    assert_eq!(code, 503);
    assert_eq!(body, "not ready: redis");
}

#[actix_web::test]
#[serial]
async fn not_ready_without_postgres() {
    // `AppState::new` panics without Postgres, so the state is built on a dedicated database
    // that is stopped afterwards (the shared one is used by the other tests).
    let (postgres, db) = start_postgres_container().await;
    let pg_url = format!(
        "postgres://postgres:postgres@{}:{}/postgres",
        db.host, db.port
    );
    let (_redis, redis) = start_redis_container().await;
    let state = AppState::new(redis.url.clone(), pg_url).await;
    postgres.stop().await.expect("stop the Postgres container");

    let (code, body) = ready(state).await;
    assert_eq!(code, 503);
    assert_eq!(body, "not ready: postgres");
}

#[actix_web::test]
#[serial]
async fn probes_are_not_mounted_under_api() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let user = ctx.user("Prober", None).await;
    assert_eq!(status(&app, get("/api/health", user)).await, 404);
    assert_eq!(status(&app, get("/api/ready", user)).await, 404);
}
