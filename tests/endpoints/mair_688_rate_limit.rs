//! The `/api` scope is limited per user by the lib's limiter, mounted like `main.rs` (token
//! checked first, then the budget), and the contract documents the `429`.

use std::num::NonZeroU32;

use actix_web::middleware::from_fn;
use actix_web::test::{call_service, init_service};
use actix_web::{web, App};
use mairie360_api_lib::security::{
    rate_limit_middleware, JwtMiddleware, RateLimitConfig, RateLimiter,
};
use project_api::endpoints::swagger::ApiDoc;
use utoipa::OpenApi;

use super::{get, TestContext};

fn limiter(burst: u32) -> RateLimiter {
    RateLimiter::new(RateLimitConfig {
        per_second: NonZeroU32::MIN,
        burst: NonZeroU32::new(burst).unwrap(),
        trust_forwarded: false,
    })
}

#[tokio::test]
async fn mair_688_each_user_has_their_own_budget_and_a_429_carries_retry_after() {
    let ctx = TestContext::new().await;
    let first = ctx.user("RateFirst", Some("User")).await;
    let second = ctx.user("RateSecond", Some("User")).await;
    let app = init_service(
        App::new()
            .app_data(ctx.state.clone())
            .app_data(limiter(2))
            .service(
                web::scope("/api")
                    .wrap(from_fn(rate_limit_middleware))
                    .wrap(JwtMiddleware)
                    .configure(project_api::endpoints::config),
            ),
    )
    .await;

    for _ in 0..2 {
        let res = call_service(&app, get("/api/v1/projects/", first).to_request()).await;
        assert_eq!(res.status(), 200);
    }
    let limited = call_service(&app, get("/api/v1/projects/", first).to_request()).await;
    assert_eq!(limited.status(), 429);
    let retry_after: u64 = limited
        .headers()
        .get("retry-after")
        .unwrap()
        .to_str()
        .unwrap()
        .parse()
        .unwrap();
    assert!(retry_after >= 1);
    assert_eq!(
        limited.headers().get("content-type").unwrap(),
        "text/plain; charset=utf-8"
    );

    // Another user is not affected by the first one's burst.
    let res = call_service(&app, get("/api/v1/projects/", second).to_request()).await;
    assert_eq!(res.status(), 200);
}

#[tokio::test]
async fn mair_688_a_request_without_token_is_refused_before_it_costs_budget() {
    let ctx = TestContext::new().await;
    let user = ctx.user("RateToken", Some("User")).await;
    let app = init_service(
        App::new()
            .app_data(ctx.state.clone())
            .app_data(limiter(1))
            .service(
                web::scope("/api")
                    .wrap(from_fn(rate_limit_middleware))
                    .wrap(JwtMiddleware)
                    .configure(project_api::endpoints::config),
            ),
    )
    .await;
    let anonymous = || actix_web::test::TestRequest::get().uri("/api/v1/projects/");
    for _ in 0..3 {
        assert_eq!(super::status(&app, anonymous()).await, 401);
    }
    let res = call_service(&app, get("/api/v1/projects/", user).to_request()).await;
    assert_eq!(res.status(), 200);
}

#[test]
fn mair_688_every_secured_operation_documents_the_429() {
    let document = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let mut checked = 0;
    for (template, methods) in document["paths"].as_object().unwrap() {
        for (method, operation) in methods.as_object().unwrap() {
            if operation["security"]
                .as_array()
                .is_some_and(|s| !s.is_empty())
            {
                let response = &operation["responses"]["429"];
                assert!(
                    response["content"]["text/plain"]["example"].is_string(),
                    "{method} {template} does not document the 429"
                );
                checked += 1;
            }
        }
    }
    assert!(checked > 0);
}
