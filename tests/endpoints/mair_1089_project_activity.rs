//! MAIR-1089: the activity of a project (its comments and the history of its tasks, archived ones
//! included, newest first), its comments (anyone who sees it), and the bulk deletion of projects
//! (each checked and deleted on its own).

use serde_json::{json, Value};

use super::access::scenario;
use super::{get, json, patch, post, status, TestContext};
use crate::init_app;

fn kinds(page: &Value) -> Vec<String> {
    page["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["kind"].as_str().unwrap().to_string())
        .collect()
}

#[actix_web::test]
async fn mair_1089_viewers_read_the_activity_and_comment() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let s = scenario(&ctx, &app).await;
    let activity = format!("{}activity/", s.project());
    let comments = format!("{}comments/", s.project());

    // The creation of the scenario's task.
    let page = json(&app, get(&activity, s.member)).await;
    assert_eq!(page["total"], 1, "{page}");
    let created = &page["items"][0];
    assert_eq!(created["kind"], "task_created");
    assert_eq!(created["task_id"], s.task_id);
    assert_eq!(created["task_title"], "Consulter les riverains");
    assert_eq!(created["author_id"], s.manager);
    assert_eq!(created["label"], "Task created");
    assert!(created["message"].is_null());

    // A plain member comments; the message is trimmed and signed by the caller.
    let comment = json(
        &app,
        post(
            &comments,
            s.member,
            json!({ "message": "  Réunion > 18 h  " }),
        ),
    )
    .await;
    assert_eq!(comment["kind"], "comment");
    assert_eq!(comment["message"], "Réunion > 18 h");
    assert_eq!(comment["author_id"], s.member);
    assert!(comment["task_id"].is_null());
    assert_eq!(comment["label"], "Comment");
    assert!(comment["created_at"].as_str().unwrap().ends_with('Z'));

    // The assignee completes the task, which is archived: its history stays in the activity.
    let done = patch(&s.task(), s.assignee, json!({ "status": "Completed" }));
    assert_eq!(status(&app, done).await, 204);
    let page = json(&app, get(&activity, s.manager)).await;
    assert_eq!(page["total"], 3, "{page}");
    assert_eq!(kinds(&page), ["status_changed", "comment", "task_created"]);
    assert_eq!(page["items"][0]["author_id"], s.assignee);
    assert!(page["items"][0]["label"]
        .as_str()
        .unwrap()
        .starts_with("Status changed: "));
    let paged = json(&app, get(&format!("{activity}?limit=1&offset=1"), s.member)).await;
    assert_eq!(paged["total"], 3);
    assert_eq!(kinds(&paged), ["comment"]);

    // Refusals.
    let long = "a".repeat(2001);
    for body in [
        json!({ "message": "" }),
        json!({ "message": "  \n " }),
        json!({ "message": long }),
        json!({ "message": "a\u{0}b" }),
        json!({}),
    ] {
        assert_eq!(
            status(&app, post(&comments, s.member, body.clone())).await,
            400,
            "{body}"
        );
    }
    for outsider in [s.outsider, s.outsider_manager] {
        assert_eq!(status(&app, get(&activity, outsider)).await, 404);
        let body = json!({ "message": "Intrus" });
        assert_eq!(status(&app, post(&comments, outsider, body)).await, 404);
    }
    assert_eq!(
        status(&app, get(&format!("{activity}?limit=-1"), s.member)).await,
        400
    );
    let unknown = format!("/api/v1/projects/{}/activity/", i32::MAX);
    assert_eq!(status(&app, get(&unknown, s.manager)).await, 404);
    let page = json(&app, get(&activity, s.manager)).await;
    assert_eq!(page["total"], 3, "nothing written by the refusals");
}

#[actix_web::test]
async fn mair_1089_bulk_deletion_checks_each_project_on_its_own() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let s = scenario(&ctx, &app).await;
    let create =
        |owner: u64, name: &'static str| post("/api/v1/projects/", owner, json!({ "name": name }));
    let first = json(&app, create(s.manager, "Premier")).await["project_id"]
        .as_u64()
        .unwrap();
    let second = json(&app, create(s.manager, "Second")).await["project_id"]
        .as_u64()
        .unwrap();
    let foreign = json(&app, create(s.outsider_manager, "Ailleurs")).await["project_id"]
        .as_u64()
        .unwrap();
    let bulk = "/api/v1/projects/bulk-delete/";

    // A plain member sees the project but may not delete it.
    let refused = json(&app, post(bulk, s.member, json!({ "ids": [s.project_id] }))).await;
    assert_eq!(
        refused["results"],
        json!([{ "id": s.project_id, "status": "forbidden" }])
    );
    assert_eq!(status(&app, get(&s.project(), s.member)).await, 200);

    // One forbidden or unknown project never blocks the others; a repeated id is handled once.
    let ghost = u64::try_from(i32::MAX).unwrap();
    let done = json(
        &app,
        post(
            bulk,
            s.manager,
            json!({ "ids": [first, foreign, first, ghost, second] }),
        ),
    )
    .await;
    assert_eq!(
        done["results"],
        json!([
            { "id": first, "status": "deleted" },
            { "id": foreign, "status": "not_found" },
            { "id": ghost, "status": "not_found" },
            { "id": second, "status": "deleted" }
        ])
    );
    for gone in [first, second] {
        let path = format!("/api/v1/projects/{gone}/");
        assert_eq!(status(&app, get(&path, s.manager)).await, 404);
    }
    let kept = format!("/api/v1/projects/{foreign}/");
    assert_eq!(status(&app, get(&kept, s.outsider_manager)).await, 200);

    // Bad bodies.
    let too_many: Vec<u64> = (1..=101).collect();
    for body in [
        json!({ "ids": [] }),
        json!({ "ids": too_many }),
        json!({ "ids": [-1] }),
        json!({}),
    ] {
        assert_eq!(status(&app, post(bulk, s.manager, body)).await, 400);
    }
}
