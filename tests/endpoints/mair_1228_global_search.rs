//! MAIR-1228: `GET /api/v1/search`, Project's part of the global search of the shared header
//! (projects and tasks the caller sees). Each test isolates its rows with a unique marker used as
//! the term.

use actix_web::test::TestRequest;
use serde_json::{json, Value};

use super::{get, json, post, status, status_raw, TestContext};
use crate::common::fixtures::unique_suffix;
use crate::init_app;

/// A marker made of letters only, unique to the test.
fn marker(prefix: &str) -> String {
    let digits = unique_suffix();
    let letters: String = digits
        .chars()
        .filter(char::is_ascii_digit)
        .map(|d| char::from(b'a' + (d as u8 - b'0')))
        .collect();
    format!("{prefix}{letters}")
}

fn items(body: &Value) -> Vec<Value> {
    body["items"].as_array().expect("items").clone()
}

#[actix_web::test]
async fn mair_1228_finds_a_visible_project_and_its_task_not_another_users() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let owner = ctx.user("Owner", Some("Responsable")).await;
    let stranger = ctx.user("Stranger", None).await;
    let term = marker("cantine");

    let project_id = json(
        &app,
        post(
            "/api/v1/projects/",
            owner,
            json!({ "name": format!("{term} école"), "description": "Menus bio et circuits courts" }),
        ),
    )
    .await["project_id"]
        .as_u64()
        .unwrap();
    let task_id = json(
        &app,
        post(
            &format!("/api/v1/projects/{project_id}/tasks/"),
            owner,
            json!({ "name": format!("Devis {term}"), "fields": [], "assigned_to": owner }),
        ),
    )
    .await["task_id"]
        .as_u64()
        .unwrap();

    let found = items(&json(&app, get(&format!("/api/v1/search?q={term}"), owner)).await);
    assert_eq!(found.len(), 2, "{found:?}");
    // The project title starts with the term: it comes first.
    assert_eq!(found[0]["kind"], "project");
    assert_eq!(found[0]["id"], project_id);
    assert_eq!(found[0]["excerpt"], "Menus bio et circuits courts");
    assert!(found[0]["parent_id"].is_null());
    assert_eq!(found[1]["kind"], "task");
    assert_eq!(found[1]["id"], task_id);
    assert_eq!(found[1]["parent_id"], project_id);
    assert!(found[1]["updated_at"].as_str().unwrap().ends_with('Z'));

    // The description matches too.
    let by_description = items(&json(&app, get("/api/v1/search?q=circuits%20courts", owner)).await);
    assert!(by_description
        .iter()
        .any(|item| item["id"] == project_id && item["kind"] == "project"));

    // Another agent sees neither.
    let hidden = items(&json(&app, get(&format!("/api/v1/search?q={term}"), stranger)).await);
    assert!(hidden.is_empty(), "{hidden:?}");
}

#[actix_web::test]
async fn mair_1228_limit_cuts_and_wildcards_match_themselves() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let owner = ctx.user("Owner", Some("Responsable")).await;
    let term = marker("voirie");
    for index in 0..3 {
        let created = post(
            "/api/v1/projects/",
            owner,
            json!({ "name": format!("Lot {index} {term}") }),
        );
        assert_eq!(status(&app, created).await, 200);
    }

    let found = items(
        &json(
            &app,
            get(&format!("/api/v1/search?q={term}&limit=2"), owner),
        )
        .await,
    );
    assert_eq!(found.len(), 2, "{found:?}");
    // `%_` would match every project if the wildcards were not escaped.
    let wildcards = items(&json(&app, get("/api/v1/search?q=%25_", owner)).await);
    assert!(wildcards.is_empty(), "{wildcards:?}");
}

#[actix_web::test]
async fn mair_1228_rejects_a_short_term_a_bad_limit_and_no_session() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let owner = ctx.user("Owner", Some("Responsable")).await;
    for query in ["?q=a", "?q=%20a%20", "", "?q=ab&limit=21", "?q=ab&limit=0"] {
        let refused = get(&format!("/api/v1/search{query}"), owner);
        assert_eq!(status(&app, refused).await, 400, "{query}");
    }
    let anonymous = TestRequest::get()
        .uri("/api/v1/search?q=ecole")
        .to_request();
    assert_eq!(status_raw(&app, anonymous).await, 401);
}
