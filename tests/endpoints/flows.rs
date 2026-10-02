//! Allowed paths: a manager runs a project from creation to deletion through every route.

use serde_json::json;
use serial_test::serial;

use super::access::scenario;
use super::{delete, get, json, patch, post, status, TestContext};
use crate::init_app;

#[actix_web::test]
#[serial]
async fn a_manager_runs_a_project_end_to_end() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let s = scenario(&ctx, &app).await;
    let p = s.project();
    let t = s.task();

    let list = json(&app, get("/api/v1/projects/?limit=500", s.manager)).await;
    assert!(list["projects"]
        .as_array()
        .unwrap()
        .iter()
        .any(|project| project["id"] == s.project_id));

    let renamed = patch(
        &p,
        s.manager,
        json!({ "name": "Place du marché", "description": "Budget > 10 000 €" }),
    );
    assert_eq!(status(&app, renamed).await, 204);
    let project = json(&app, get(&p, s.manager)).await;
    assert_eq!(project["project"]["name"], "Place du marché");
    // `<` and `>` are ordinary text (MAIR-426), stored and returned as sent.
    assert_eq!(project["project"]["description"], "Budget > 10 000 €");
    assert_eq!(project["users"].as_array().unwrap().len(), 2);

    let task_change = patch(
        &t,
        s.manager,
        json!({ "priority": "Urgent", "description": "Avant le 15 octobre", "assigned_to": s.member }),
    );
    assert_eq!(status(&app, task_change).await, 204);
    let tasks = json(&app, get(&format!("{p}tasks/"), s.manager)).await;
    assert_eq!(tasks["total"], 1);

    let comment = post(
        &format!("{t}comments"),
        s.manager,
        json!({ "message": "Validé <3 -> on lance" }),
    );
    let created = json(&app, comment).await;
    assert_eq!(created["message"], "Validé <3 -> on lance");
    let collaboration = json(&app, get(&format!("{t}collaboration"), s.manager)).await;
    assert_eq!(collaboration["comments_total"], 1);
    assert!(collaboration["history_total"].as_u64().unwrap() >= 2);

    let users = json(&app, get(&format!("{p}users/"), s.manager)).await;
    assert_eq!(users["users"].as_array().unwrap().len(), 2);
    let again = post(
        &format!("{p}users/"),
        s.manager,
        json!({ "user_id": s.member }),
    );
    assert_eq!(status(&app, again).await, 409);
    let unknown = post(
        &format!("{p}users/"),
        s.manager,
        json!({ "user_id": i32::MAX }),
    );
    assert_eq!(status(&app, unknown).await, 404);
    let removed = delete(&format!("{p}users/{}/", s.assignee), s.manager);
    assert_eq!(status(&app, removed).await, 204);

    assert_eq!(status(&app, delete(&t, s.manager)).await, 204);
    assert_eq!(status(&app, delete(&t, s.manager)).await, 404);
    let close = patch(&format!("{p}close"), s.manager, json!({}));
    assert_eq!(status(&app, close).await, 200);
    let project = json(&app, get(&p, s.manager)).await;
    assert_eq!(project["project"]["status"], "Completed");

    assert_eq!(status(&app, delete(&p, s.manager)).await, 204);
    assert_eq!(status(&app, get(&p, s.manager)).await, 404);
}

#[actix_web::test]
#[serial]
async fn removing_a_member_unassigns_their_tasks_in_the_same_write() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let s = scenario(&ctx, &app).await;
    let p = s.project();
    let tasks = json(&app, get(&format!("{p}tasks/"), s.manager)).await;
    assert_eq!(tasks["tasks"][0]["assigned_to"], s.assignee);

    let removed = delete(&format!("{p}users/{}/", s.assignee), s.manager);
    assert_eq!(status(&app, removed).await, 204);

    let tasks = json(&app, get(&format!("{p}tasks/"), s.manager)).await;
    let task = &tasks["tasks"][0];
    assert_eq!(task["id"], s.task_id);
    assert!(task["assigned_to"].is_null(), "{task}");
    // The former assignee lost every access to the task.
    let t = s.task();
    let comment = post(
        &format!("{t}comments"),
        s.assignee,
        json!({ "message": "Encore là ?" }),
    );
    assert_eq!(status(&app, comment).await, 404);
}
