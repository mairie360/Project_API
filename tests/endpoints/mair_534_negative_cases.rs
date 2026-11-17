//! MAIR-534: business negative cases that only had their nominal case: a malformed id on the archived-tasks
//! list, closing a project twice, and the bounds of `limit` on the lists.

use serde_json::json;

use super::access::scenario;
use super::{get, json, patch, post, status, TestContext};
use crate::init_app;

#[actix_web::test]
async fn archived_tasks_refuse_a_malformed_project_id_and_an_unknown_project() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let s = scenario(&ctx, &app).await;

    for id in ["abc", "-1", "1.5"] {
        assert_eq!(
            status(
                &app,
                get(&format!("/api/v1/projects/{id}/archived-tasks/"), s.manager)
            )
            .await,
            400,
            "{id} is not a project id"
        );
    }
    assert_eq!(
        status(
            &app,
            get("/api/v1/projects/2147483000/archived-tasks/", s.manager)
        )
        .await,
        404
    );
}

#[actix_web::test]
async fn closing_a_project_twice_is_idempotent() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let s = scenario(&ctx, &app).await;
    let close = format!("{}close", s.project());

    assert_eq!(status(&app, patch(&close, s.manager, json!({}))).await, 200);
    assert_eq!(status(&app, patch(&close, s.manager, json!({}))).await, 200);
    let read = json(&app, get(&s.project(), s.manager)).await;
    assert_eq!(read["project"]["status"], "Completed");
}

#[actix_web::test]
async fn the_list_limit_is_bounded_and_refused_when_it_is_not_an_integer() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let s = scenario(&ctx, &app).await;
    // A second project, so that a limit of one is visible.
    assert_eq!(
        status(
            &app,
            post("/api/v1/projects/", s.manager, json!({ "name": "Second" }))
        )
        .await,
        200
    );

    // `limit=0` is raised to one item, never an empty or a failing page.
    let one = json(&app, get("/api/v1/projects/?limit=0", s.manager)).await;
    assert_eq!(one["projects"].as_array().unwrap().len(), 1);
    let one_task_page = status(&app, get("/api/v1/tasks/?limit=0", s.manager)).await;
    assert_eq!(one_task_page, 200);
    // A limit above the maximum (500) is served, capped.
    assert_eq!(
        status(&app, get("/api/v1/projects/?limit=100000", s.manager)).await,
        200
    );
    for limit in ["-1", "abc", "1.5"] {
        for list in ["/api/v1/projects/", "/api/v1/tasks/"] {
            assert_eq!(
                status(&app, get(&format!("{list}?limit={limit}"), s.manager)).await,
                400,
                "{list}?limit={limit}"
            );
        }
    }
}
