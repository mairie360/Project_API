//! MAIR-680: Core is the only signer of the session JWTs. This API holds Core's public key
//! (`JWT_PUBLIC_KEY`) and no `JWT_SECRET`: it accepts the `EdDSA` tokens signed with Core's key and
//! refuses every HS256 token, whatever secret signed it, so a service that leaks its configuration
//! can no longer forge an admin token for this API.

use actix_web::test::TestRequest;
use jsonwebtoken::{encode, EncodingKey, Header};
use mairie360_api_lib::jwt_manager::Claims;
use serde_json::json;
use serial_test::serial;

use super::{delete, get, json, now, post, status, TestContext};
use crate::init_app;

/// HS256 token of `user_id` signed with `secret`: what any holder of the old shared secret signs.
fn hs256_token(user_id: u64, secret: &[u8]) -> String {
    encode(
        &Header::default(),
        &Claims::new(&user_id.to_string(), "Admin", now() + 3600),
        &EncodingKey::from_secret(secret),
    )
    .unwrap()
}

#[actix_web::test]
#[serial]
async fn mair_680_runs_without_jwt_secret() {
    let _ctx = TestContext::new().await;
    assert!(std::env::var_os("JWT_SECRET").is_none());
    assert!(std::env::var_os("JWT_PRIVATE_KEY").is_none());
    assert!(std::env::var_os("JWT_PUBLIC_KEY").is_some());
}

#[actix_web::test]
#[serial]
async fn mair_680_eddsa_token_signed_with_core_key_is_accepted() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let user = ctx.user("Eddsa", None).await;

    assert_eq!(status(&app, get("/api/v1/projects/", user)).await, 200);
}

#[actix_web::test]
#[serial]
async fn mair_680_hs256_token_is_refused_without_jwt_secret() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let admin = ctx.user("Hs256", Some("Admin")).await;

    // The secrets the stacks and the test suites used to share, and any other one.
    for secret in [
        b"project-api-endpoint-tests-secret-0123456789".as_slice(),
        b"b\"secret\"",
        b"any-secret-of-a-compromised-service-0123456789",
    ] {
        for uri in ["/api/v1/projects/", "/api/v1/tasks/"] {
            let request = TestRequest::get().uri(uri).insert_header((
                "Authorization",
                format!("Bearer {}", hs256_token(admin, secret)),
            ));
            assert_eq!(status(&app, request).await, 401, "HS256 on {uri}");
        }
    }
}

/// Business journey with Core's tokens: an administrator creates a project, reads it back and
/// deletes it; the same administrator's HS256 token is refused on that project at every step.
#[actix_web::test]
#[serial]
async fn mair_680_admin_manages_a_project_with_core_tokens_only() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let admin = ctx.user("Journey", Some("Admin")).await;

    let created = json(
        &app,
        post(
            "/api/v1/projects/",
            admin,
            json!({
                "name": "Réfection du parvis de la mairie",
                "description": "MAIR-680",
                "status": "Todo",
                "priority": "High"
            }),
        ),
    )
    .await;
    let uri = format!("/api/v1/projects/{}/", created["project_id"]);

    let project = json(&app, get(&uri, admin)).await;
    assert_eq!(
        project["project"]["name"],
        "Réfection du parvis de la mairie"
    );

    let legacy = format!("Bearer {}", hs256_token(admin, b"b\"secret\""));
    for request in [
        TestRequest::get().uri(&uri),
        TestRequest::delete().uri(&uri),
    ] {
        let request = request.insert_header(("Authorization", legacy.as_str()));
        assert_eq!(status(&app, request).await, 401);
    }

    assert_eq!(status(&app, delete(&uri, admin)).await, 204);
    assert_eq!(status(&app, get(&uri, admin)).await, 404);
}
