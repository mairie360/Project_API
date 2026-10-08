//! Handler-level tests: the real `/api` tree of `main.rs` (`JwtMiddleware` + `endpoints::config`)
//! served by `actix_web::test` against the shared test database, with JWTs signed by the test
//! secret. They cover what the query tests cannot see: the access checks, the validation and the
//! status codes of every handler.

pub mod access;
pub mod flows;
pub mod service;
pub mod token_refusals;

use std::sync::Once;

use actix_web::body::MessageBody;
use actix_web::test::TestRequest;
use actix_web::web;
use mairie360_api_lib::jwt_manager::generate_jwt;
use mairie360_api_lib::smart_db::SmartDatabase;
use mairie360_api_lib::state::AppState;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use serde_json::Value;

use crate::common::fixtures::create_user;

/// Signs a JWT for `user_id` with the test secret.
pub fn jwt_for(user_id: u64) -> String {
    set_jwt_env();
    let token = generate_jwt(&user_id.to_string(), "User").expect("failed to sign test JWT");
    format!("Bearer {token}")
}

fn set_jwt_env() {
    static JWT_ENV: Once = Once::new();
    JWT_ENV.call_once(|| {
        std::env::set_var("JWT_SECRET", "project-api-endpoint-tests-secret-0123456789");
        std::env::set_var("JWT_TIMEOUT", "3600");
    });
}

/// The app state handed to the test app; its `SmartDatabase` also seeds the fixtures.
pub struct TestContext {
    pub state: web::Data<AppState>,
}

impl TestContext {
    pub async fn new() -> Self {
        set_jwt_env();
        let (_container, pg_url) = get_shared_db().await;
        // Redis is not needed: no query view declares a cache key and the test JWTs carry no
        // session id, so the revocation list is never read.
        let state = AppState::new("redis://127.0.0.1:6379".to_string(), pg_url.to_string()).await;
        Self {
            state: web::Data::new(state),
        }
    }

    pub fn db(&self) -> &SmartDatabase {
        self.state.get_smart_db()
    }

    pub async fn user(&self, name: &str, role: Option<&str>) -> u64 {
        create_user(self.db(), name, role).await
    }
}

/// Builds the app exactly like `main.rs` mounts `/api`.
#[macro_export]
macro_rules! init_app {
    ($ctx:expr) => {{
        actix_web::test::init_service(
            actix_web::App::new().app_data($ctx.state.clone()).service(
                actix_web::web::scope("/api")
                    .wrap(mairie360_api_lib::security::JwtMiddleware)
                    .configure(project_api::endpoints::config),
            ),
        )
        .await
    }};
}

/// Services built by [`init_app!`].
pub trait TestApp<B>:
    actix_web::dev::Service<
    actix_http::Request,
    Response = actix_web::dev::ServiceResponse<B>,
    Error = actix_web::Error,
>
{
}

impl<S, B> TestApp<B> for S where
    S: actix_web::dev::Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse<B>,
        Error = actix_web::Error,
    >
{
}

/// Sends a request and returns its status, whether the handler answered or a middleware rejected
/// it with an error.
pub async fn status<B>(app: &impl TestApp<B>, request: TestRequest) -> u16 {
    status_raw(app, request.to_request()).await
}

/// [`status`] for an already built request.
pub async fn status_raw<B>(app: &impl TestApp<B>, request: actix_http::Request) -> u16 {
    match actix_web::test::try_call_service(app, request).await {
        Ok(response) => response.status().as_u16(),
        Err(error) => error.as_response_error().status_code().as_u16(),
    }
}

/// Sends a request that must succeed and returns its JSON body.
pub async fn json<B: MessageBody>(app: &impl TestApp<B>, request: TestRequest) -> Value {
    let response = actix_web::test::call_service(app, request.to_request()).await;
    assert!(
        response.status().is_success(),
        "expected a success, got {}",
        response.status()
    );
    actix_web::test::read_body_json(response).await
}

/// `TestRequest` with the caller's JWT.
pub fn as_user(request: TestRequest, user_id: u64) -> TestRequest {
    request.insert_header(("Authorization", jwt_for(user_id)))
}

pub fn get(uri: &str, user_id: u64) -> TestRequest {
    as_user(TestRequest::get().uri(uri), user_id)
}

pub fn delete(uri: &str, user_id: u64) -> TestRequest {
    as_user(TestRequest::delete().uri(uri), user_id)
}

pub fn post(uri: &str, user_id: u64, body: serde_json::Value) -> TestRequest {
    as_user(TestRequest::post().uri(uri).set_json(body), user_id)
}

pub fn patch(uri: &str, user_id: u64, body: serde_json::Value) -> TestRequest {
    as_user(TestRequest::patch().uri(uri).set_json(body), user_id)
}
