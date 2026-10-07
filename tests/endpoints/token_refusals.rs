//! Token refusals on every secured operation (MAIR-474), driven by the published contract
//! (`ApiDoc`): an operation is covered as soon as it is documented, and the sweep fails if one
//! accepts a token it should not.
//!
//! The tokens are the ones an attacker can build without `JWT_SECRET`: none at all, another
//! scheme, garbage, a token signed with another secret, an expired one, `alg: none`, a payload
//! swapped under a valid signature, an asymmetric algorithm (the Keycloak path, disabled here),
//! plus well-signed tokens for an unknown and for an archived account.

use actix_web::http::Method;
use actix_web::test::TestRequest;
use jsonwebtoken::{encode, EncodingKey, Header};
use mairie360_api_lib::jwt_manager::{get_jwt_secret, Claims};
use project_api::endpoints::swagger::ApiDoc;
use serial_test::serial;
use utoipa::OpenApi;

use super::{jwt_for, status, TestContext};
use crate::common::fixtures::archive_user;
use crate::init_app;

/// Every operation that declares the `jwt` scheme, as `(method, uri)`, path parameters set to `id`.
fn secured_operations(id: u64) -> Vec<(Method, String)> {
    let document = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let mut operations = Vec::new();
    for (template, methods) in document["paths"].as_object().unwrap() {
        let uri = template
            .split('/')
            .map(|segment| {
                if segment.starts_with('{') {
                    id.to_string()
                } else {
                    segment.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("/");
        for (method, operation) in methods.as_object().unwrap() {
            let secured = operation["security"]
                .as_array()
                .is_some_and(|schemes| schemes.iter().any(|scheme| scheme.get("jwt").is_some()));
            if secured {
                operations.push((
                    Method::from_bytes(method.to_uppercase().as_bytes()).unwrap(),
                    uri.clone(),
                ));
            }
        }
    }
    operations
}

/// Unpadded base64url, the JWT encoding of each part.
fn base64url(bytes: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let n = chunk
            .iter()
            .enumerate()
            .fold(0u32, |n, (i, byte)| n | u32::from(*byte) << (16 - 8 * i));
        for i in 0..=chunk.len() {
            out.push(ALPHABET[(n >> (18 - 6 * i) & 63) as usize] as char);
        }
    }
    out
}

fn now() -> usize {
    usize::try_from(chrono::Utc::now().timestamp()).unwrap()
}

/// Header values that must all be refused with `401`, labelled for the failure message.
fn refused_headers(user_id: u64) -> Vec<(&'static str, String)> {
    let secret = get_jwt_secret().unwrap();
    let valid = Claims::new(&user_id.to_string(), "User", now() + 3600);
    let sign = |claims: &Claims, key: &[u8]| {
        encode(&Header::default(), claims, &EncodingKey::from_secret(key)).unwrap()
    };

    let genuine = sign(&valid, &secret);
    let mut parts = genuine.split('.');
    let (header, signature) = (parts.next().unwrap(), parts.nth(1).unwrap());
    let other_user = base64url(
        serde_json::to_string(&Claims::new(
            &(user_id + 1).to_string(),
            "Admin",
            now() + 3600,
        ))
        .unwrap()
        .as_bytes(),
    );
    let payload = base64url(serde_json::to_string(&valid).unwrap().as_bytes());

    vec![
        ("other scheme", format!("Basic {genuine}")),
        ("token without scheme", genuine.clone()),
        ("empty bearer", "Bearer ".to_string()),
        ("garbage", "Bearer not.a.jwt".to_string()),
        (
            "other secret",
            format!(
                "Bearer {}",
                sign(&valid, b"not-the-secret-of-this-deployment-0123")
            ),
        ),
        (
            "expired",
            format!(
                "Bearer {}",
                sign(
                    &Claims::new(&user_id.to_string(), "User", now() - 60),
                    &secret
                )
            ),
        ),
        (
            "alg none",
            format!(
                "Bearer {}.{payload}.",
                base64url(br#"{"alg":"none","typ":"JWT"}"#)
            ),
        ),
        (
            "payload swapped under a valid signature",
            format!("Bearer {header}.{other_user}.{signature}"),
        ),
        (
            "asymmetric algorithm",
            format!(
                "Bearer {}.{payload}.{signature}",
                base64url(br#"{"alg":"RS256","typ":"JWT"}"#)
            ),
        ),
    ]
}

#[actix_web::test]
#[serial]
async fn every_secured_operation_refuses_a_missing_or_forged_token() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let user = ctx.user("Holder", None).await;

    let operations = secured_operations(1);
    assert!(operations.len() >= 15, "{operations:?}");
    let headers = refused_headers(user);

    for (method, uri) in &operations {
        let anonymous = TestRequest::default().method(method.clone()).uri(uri);
        assert_eq!(
            status(&app, anonymous).await,
            401,
            "no token: {method} {uri}"
        );

        for (label, value) in &headers {
            let request = TestRequest::default()
                .method(method.clone())
                .uri(uri)
                .insert_header(("Authorization", value.as_str()))
                .set_json(serde_json::json!({}));
            assert_eq!(status(&app, request).await, 401, "{label}: {method} {uri}");
        }
    }
}

#[actix_web::test]
#[serial]
async fn every_secured_operation_refuses_an_unknown_or_archived_account() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let archived = ctx.user("Archived", Some("Admin")).await;
    archive_user(ctx.db(), archived).await;

    for (method, uri) in secured_operations(1) {
        for (label, holder) in [
            ("unknown", u64::from(i32::MAX.unsigned_abs())),
            ("archived", archived),
        ] {
            let request = TestRequest::default()
                .method(method.clone())
                .uri(&uri)
                .insert_header(("Authorization", jwt_for(holder)))
                .set_json(serde_json::json!({}));
            assert_eq!(status(&app, request).await, 404, "{label}: {method} {uri}");
        }
    }
}

/// Control of the sweeps: a genuine token of an active account passes the authentication of every
/// operation (whatever the handler answers next), so their `401`s come from the token alone.
#[actix_web::test]
#[serial]
async fn a_genuine_token_passes_the_authentication_of_every_operation() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let user = ctx.user("Genuine", None).await;

    for (method, uri) in secured_operations(1) {
        let request = TestRequest::default()
            .method(method.clone())
            .uri(&uri)
            .insert_header(("Authorization", jwt_for(user)))
            .set_json(serde_json::json!({}));
        let status = status(&app, request).await;
        assert!(status != 401 && status < 500, "{status}: {method} {uri}");
    }
}

/// The sweeps above only see the operations that declare `jwt`: an operation published under
/// `/api` without it would be skipped silently, so every one of them must declare it.
#[test]
fn every_api_operation_declares_jwt() {
    let document = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let mut unsecured = Vec::new();
    for (template, methods) in document["paths"].as_object().unwrap() {
        if !template.starts_with("/api/") {
            continue;
        }
        for (method, operation) in methods.as_object().unwrap() {
            let secured = operation["security"]
                .as_array()
                .is_some_and(|schemes| schemes.iter().any(|scheme| scheme.get("jwt").is_some()));
            if !secured {
                unsecured.push(format!("{} {template}", method.to_uppercase()));
            }
        }
    }
    assert!(
        unsecured.is_empty(),
        "published without `jwt`: {unsecured:?}"
    );
}
