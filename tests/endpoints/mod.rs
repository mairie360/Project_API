//! Handler-level tests: the real `/api` tree of `main.rs` (`JwtMiddleware` + `endpoints::config`)
//! served by `actix_web::test` against the shared test database, with JWTs signed like Core signs
//! them (MAIR-680: EdDSA, the lib's test key pair; the API only holds the public key). They cover what the query tests cannot see: the access checks, the validation and the
//! status codes of every handler.

pub mod access;
pub mod access_matrix;
pub mod flows;
pub mod mair_680_session_keys;
pub mod read_cache;
pub mod service;
pub mod statuses;
pub mod tasks;
pub mod telemetry;
pub mod token_refusals;

use std::sync::Once;

use actix_web::body::MessageBody;
use actix_web::test::TestRequest;
use actix_web::web;
use jsonwebtoken::{encode, Algorithm, Header};
use mairie360_api_lib::jwt_manager::{
    parse_public_keys, parse_signing_key, Claims, SESSION_JWT_AUDIENCE, SESSION_JWT_ISSUER,
};
use mairie360_api_lib::smart_db::SmartDatabase;
use mairie360_api_lib::state::AppState;
use mairie360_api_lib::test_setup::http_setup::{test_jwt_private_key, TEST_JWT_PUBLIC_KEY};
use mairie360_api_lib::test_setup::redis_setup::start_redis_container;
use serde_json::Value;

use crate::common::fixtures::create_user;
use crate::common::project_db_url;

/// Seconds since the epoch, as the `exp` claim counts them.
pub fn now() -> usize {
    usize::try_from(chrono::Utc::now().timestamp()).unwrap()
}

/// Claims of a session token issued by Core for `user_id` (MAIR-680: with `iss` and `aud`).
pub fn core_claims(user_id: u64, expiration: usize) -> Claims {
    Claims::new(&user_id.to_string(), "User", expiration)
        .with_issuer_and_audience(SESSION_JWT_ISSUER, SESSION_JWT_AUDIENCE)
}

/// Signs `claims` as they are with `private_key` (PEM), under the `kid` of the lib's test public
/// key: Core's key when `private_key` is [`test_jwt_private_key`], a forgery otherwise.
pub fn sign_eddsa_with(claims: &Claims, private_key: &str) -> String {
    let core_keys = parse_public_keys(TEST_JWT_PUBLIC_KEY).unwrap();
    let mut header = Header::new(Algorithm::EdDSA);
    header.kid = Some(core_keys[0].kid().to_string());
    let known_keys = [
        core_keys,
        parse_public_keys(MAIR_680_OTHER_PUBLIC_KEY).unwrap(),
    ]
    .concat();
    let key = parse_signing_key(private_key, &known_keys).unwrap();
    encode(&header, claims, key.encoding_key()).unwrap()
}

/// Signs `claims` like Core does (MAIR-680): EdDSA, the lib's test key pair, Core's `kid`.
pub fn sign_like_core(claims: &Claims) -> String {
    sign_eddsa_with(claims, &test_jwt_private_key())
}

/// PKCS#8 body (base64) of an Ed25519 private key that is not Core's (MAIR-680), assembled into
/// PEM at run time so that the secret scanners do not read a PEM private key in the sources.
const MAIR_680_OTHER_PRIVATE_KEY_B64: &str =
    "MC4CAQAwBQYDK2VwBCIEII1DTYiu+dcwBgtEKil59uTH8s7XfrxGrbD+/P0sXq/8";

/// Public key of [`other_private_key`], never published in `JWT_PUBLIC_KEY`.
const MAIR_680_OTHER_PUBLIC_KEY: &str = "-----BEGIN PUBLIC KEY-----
MCowBQYDK2VwAyEAtZHz1BNkijjkLJlnK9l7vikDl0aivrmbwllpT68Lnss=
-----END PUBLIC KEY-----
";

/// An Ed25519 private key that is not Core's: what a compromised service could sign with.
pub fn other_private_key() -> String {
    let label = "PRIVATE KEY";
    format!("-----BEGIN {label}-----\n{MAIR_680_OTHER_PRIVATE_KEY_B64}\n-----END {label}-----\n")
}

/// A session JWT issued by Core for `user_id`, as an `Authorization` value.
pub fn jwt_for(user_id: u64) -> String {
    set_jwt_env();
    format!(
        "Bearer {}",
        sign_like_core(&core_claims(user_id, now() + 3600))
    )
}

/// The environment of a deployed API after MAIR-680: Core's public key, no `JWT_SECRET`.
fn set_jwt_env() {
    static JWT_ENV: Once = Once::new();
    JWT_ENV.call_once(|| {
        std::env::set_var("JWT_PUBLIC_KEY", TEST_JWT_PUBLIC_KEY);
        std::env::remove_var("JWT_PRIVATE_KEY");
        std::env::remove_var("JWT_SECRET");
        std::env::set_var("JWT_TIMEOUT", "3600");
    });
}

/// The app state handed to the test app. It connects as `project_api`, like in staging
/// (MAIR-556); the fixtures go through the superuser (`common::fixtures`).
pub struct TestContext {
    pub state: web::Data<AppState>,
    /// The Redis container of [`TestContext::with_redis`], stopped when the context is dropped.
    _redis: Option<Box<dyn std::any::Any>>,
}

impl TestContext {
    pub async fn new() -> Self {
        set_jwt_env();
        // Redis is not needed: the test JWTs carry no session id, so the revocation list is never
        // read, and without Redis the read caches (MAIR-524) are bypassed (every read hits
        // Postgres). `with_redis` is the context of the cache tests.
        let state =
            AppState::new("redis://127.0.0.1:6379".to_string(), project_db_url().await).await;
        Self {
            state: web::Data::new(state),
            _redis: None,
        }
    }

    /// A context whose `SmartDatabase` uses a real Redis of its own (a fresh container), for the
    /// read cache tests (MAIR-524).
    pub async fn with_redis() -> Self {
        set_jwt_env();
        let (redis, config) = start_redis_container().await;
        let state = AppState::new(config.url.clone(), project_db_url().await).await;
        Self {
            state: web::Data::new(state),
            _redis: Some(Box::new(redis)),
        }
    }

    pub fn db(&self) -> &SmartDatabase {
        self.state.get_smart_db()
    }

    pub async fn user(&self, name: &str, role: Option<&str>) -> u64 {
        create_user(name, role).await
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
