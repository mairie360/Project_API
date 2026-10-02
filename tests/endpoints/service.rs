//! Probes: `/health` is liveness only, `/ready` answers 200 only when Postgres and Redis respond.

use actix_web::test::TestRequest;
use actix_web::{web, App};
use mairie360_api_lib::state::AppState;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use mairie360_api_lib::test_setup::redis_setup::start_redis_container;
use project_api::endpoints::health;
use serial_test::serial;

use super::{get, status, TestContext};
use crate::init_app;

/// Nothing listens on port 1: the connection is refused immediately.
const UNREACHABLE_REDIS: &str = "redis://127.0.0.1:1";
const UNREACHABLE_POSTGRES: &str = "postgres://postgres:password@127.0.0.1:1/mairie_360_database";

async fn ready(state: AppState) -> (u16, serde_json::Value) {
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
    (code, actix_web::test::read_body_json(response).await)
}

#[actix_web::test]
#[serial]
async fn ready_when_postgres_and_redis_answer() {
    let (_container, pg_url) = get_shared_db().await;
    let (_redis, redis) = start_redis_container().await;
    let state = AppState::new(redis.url.clone(), pg_url.to_string()).await;

    let (code, body) = ready(state).await;
    assert_eq!(code, 200);
    assert_eq!(body, serde_json::json!({ "postgres": true, "redis": true }));
}

#[actix_web::test]
#[serial]
async fn not_ready_without_redis() {
    let (_container, pg_url) = get_shared_db().await;
    let state = AppState::new(UNREACHABLE_REDIS.to_string(), pg_url.to_string()).await;

    let (code, body) = ready(state).await;
    assert_eq!(code, 503);
    assert_eq!(
        body,
        serde_json::json!({ "postgres": true, "redis": false })
    );
}

#[actix_web::test]
#[serial]
async fn not_ready_without_postgres() {
    let (_redis, redis) = start_redis_container().await;
    let state = AppState::new(redis.url.clone(), UNREACHABLE_POSTGRES.to_string()).await;

    let (code, body) = ready(state).await;
    assert_eq!(code, 503);
    assert_eq!(
        body,
        serde_json::json!({ "postgres": false, "redis": true })
    );
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

#[actix_web::test]
#[serial]
async fn api_docs_are_only_served_when_enabled() {
    use project_api::endpoints::swagger::{configure_docs, docs_enabled, API_DOCS_ENV};

    for enabled in [false, true] {
        let app =
            actix_web::test::init_service(App::new().configure(|cfg| configure_docs(cfg, enabled)))
                .await;
        let expected = if enabled { 200 } else { 404 };
        let spec = TestRequest::get().uri("/api-docs/openapi.json");
        assert_eq!(status(&app, spec).await, expected, "enabled = {enabled}");
        let ui = TestRequest::get().uri("/swagger-ui/index.html");
        assert_eq!(status(&app, ui).await, expected, "enabled = {enabled}");
    }

    std::env::remove_var(API_DOCS_ENV);
    assert!(!docs_enabled(), "off by default");
    for (value, expected) in [("true", true), ("1", true), ("TRUE", true), ("no", false)] {
        std::env::set_var(API_DOCS_ENV, value);
        assert_eq!(docs_enabled(), expected, "{API_DOCS_ENV}={value}");
    }
    std::env::remove_var(API_DOCS_ENV);
}
