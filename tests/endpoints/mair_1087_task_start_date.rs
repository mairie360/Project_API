//! MAIR-1087: a task has an optional `start_date`, accepted at creation and by `PATCH` (absent
//! keeps, `null` clears), returned by every task read, and never after the due date (`400`,
//! checked by the handler and by the database for the stored date).

use serde_json::json;

use super::access::scenario;
use super::{get, json, patch, post, status, TestContext};
use crate::init_app;

#[actix_web::test]
async fn mair_1087_a_task_starts_before_it_is_due() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let s = scenario(&ctx, &app).await;
    let tasks = format!("{}tasks/", s.project());

    let created = json(
        &app,
        post(
            &tasks,
            s.manager,
            json!({ "name": "Travaux", "fields": [], "start_date": "2026-10-01",
                    "due_date": "2026-10-15", "assigned_to": s.assignee }),
        ),
    )
    .await;
    let task_id = created["task_id"].as_u64().unwrap();
    let task = format!("{tasks}{task_id}/");
    assert_eq!(
        json(&app, get(&task, s.member)).await["start_date"],
        "2026-10-01T00:00:00Z"
    );
    let page = json(&app, get(&tasks, s.member)).await;
    let listed = page["tasks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["id"] == task_id)
        .unwrap()
        .clone();
    assert_eq!(listed["start_date"], "2026-10-01T00:00:00Z");
    let project = json(&app, get(&s.project(), s.member)).await;
    assert!(project["tasks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|t| t["id"] == task_id && t["start_date"] == "2026-10-01T00:00:00Z"));
    let mine = json(
        &app,
        get(
            &format!("/api/v1/tasks/?assigned_to={}", s.assignee),
            s.assignee,
        ),
    )
    .await;
    assert!(mine["tasks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|t| t["id"] == task_id && t["start_date"] == "2026-10-01T00:00:00Z"));
    // A task without start date reads `null`.
    assert_eq!(
        json(&app, get(&s.task(), s.member)).await.get("start_date"),
        Some(&serde_json::Value::Null)
    );

    // Start after due: refused at creation, by a PATCH carrying both, or one against the stored
    // other one (the database check, mapped to 400).
    let late =
        json!({ "name": "X", "fields": [], "start_date": "2026-10-16", "due_date": "2026-10-15" });
    assert_eq!(status(&app, post(&tasks, s.manager, late)).await, 400);
    for body in [
        json!({ "start_date": "2026-11-01", "due_date": "2026-10-31" }),
        json!({ "start_date": "2026-10-16T00:00:01Z" }),
        json!({ "due_date": "2026-09-30" }),
        json!({ "start_date": "01/10/2026" }),
    ] {
        assert_eq!(
            status(&app, patch(&task, s.manager, body.clone())).await,
            400,
            "{body}"
        );
    }
    assert_eq!(
        json(&app, get(&task, s.manager)).await["start_date"],
        "2026-10-01T00:00:00Z"
    );

    // Moved within the bounds, kept by a PATCH without it, cleared by `null`.
    let moved = patch(
        &task,
        s.manager,
        json!({ "start_date": "2026-10-15T00:00:00Z" }),
    );
    assert_eq!(status(&app, moved).await, 204);
    let renamed = patch(&task, s.manager, json!({ "name": "Travaux 2" }));
    assert_eq!(status(&app, renamed).await, 204);
    assert_eq!(
        json(&app, get(&task, s.manager)).await["start_date"],
        "2026-10-15T00:00:00Z"
    );
    let cleared = patch(&task, s.manager, json!({ "start_date": null }));
    assert_eq!(status(&app, cleared).await, 204);
    let read = json(&app, get(&task, s.manager)).await;
    assert_eq!(read.get("start_date"), Some(&serde_json::Value::Null));
    assert_eq!(read["due_date"], "2026-10-15T00:00:00Z");

    // The assignee may only change the status, not the start date.
    let refused = patch(&task, s.assignee, json!({ "start_date": "2026-10-02" }));
    assert_eq!(status(&app, refused).await, 403);
}
