//! `GET /api/v1/tasks/` (MAIR-524): the tasks visible to the caller across every project, in one
//! call, filtered and paged; nothing of a project out of sight.

use actix_web::test::TestRequest;
use serde_json::{json, Value};
use serial_test::serial;

use super::access::scenario;
use super::{get, json, patch, post, status, TestContext};
use crate::init_app;

fn task_ids(page: &Value) -> Vec<u64> {
    page["tasks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|task| task["id"].as_u64().unwrap())
        .collect()
}

/// The pending tasks of an assignee come with their project, in one call, due date first; the
/// same query by someone who does not see the project lists nothing.
#[actix_web::test]
#[serial]
async fn the_pending_tasks_of_a_user_are_read_in_one_call() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let s = scenario(&ctx, &app).await;
    let dated = json(
        &app,
        post(
            &format!("/api/v1/projects/{}/tasks/", s.project_id),
            s.manager,
            json!({
                "name": "Commander le mobilier", "fields": [], "assigned_to": s.assignee,
                "due_date": "2026-10-20T08:00:00Z", "priority": "High"
            }),
        ),
    )
    .await["task_id"]
        .as_u64()
        .unwrap();

    let uri = format!("/api/v1/tasks/?assigned_to={}", s.assignee);
    for viewer in [s.assignee, s.manager, s.member] {
        let page = json(&app, get(&uri, viewer)).await;
        assert_eq!(task_ids(&page), vec![dated, s.task_id], "viewer {viewer}");
        assert_eq!(page["total"], 2);
        let first = &page["tasks"][0];
        assert_eq!(first["project_id"], s.project_id);
        assert_eq!(first["project_name"], "Réfection de la place du marché");
        assert_eq!(first["title"], "Commander le mobilier");
        assert_eq!(first["status"], "Todo");
        assert_eq!(first["priority"], "High");
        assert_eq!(first["due_date"], "2026-10-20T08:00:00Z");
        assert_eq!(first["assigned_to"], s.assignee);
        assert!(first["archived_at"].is_null());
        assert_eq!(first["description"], "");
        assert!(first["fields"].as_array().unwrap().is_empty());
        assert!(page["tasks"][1]["due_date"].is_null(), "undated last");
    }
    for outsider in [s.outsider, s.outsider_manager] {
        let page = json(&app, get(&uri, outsider)).await;
        assert_eq!(page["total"], 0, "outsider {outsider}");
        assert!(task_ids(&page).is_empty());
        let all = json(&app, get("/api/v1/tasks/?limit=500", outsider)).await;
        assert!(!task_ids(&all).contains(&s.task_id));
    }

    // Filters: due date, status, page.
    let due = json(
        &app,
        get(
            &format!("{uri}&due_before=2026-10-31T00:00:00Z"),
            s.assignee,
        ),
    )
    .await;
    assert_eq!(task_ids(&due), vec![dated]);
    let second = json(&app, get(&format!("{uri}&limit=1&offset=1"), s.assignee)).await;
    assert_eq!(task_ids(&second), vec![s.task_id]);
    assert_eq!(second["total"], 2);

    let started = patch(&s.task(), s.assignee, json!({ "status": "InProgress" }));
    assert_eq!(status(&app, started).await, 204);
    let in_progress = json(&app, get(&format!("{uri}&status=InProgress"), s.assignee)).await;
    assert_eq!(task_ids(&in_progress), vec![s.task_id]);
    let todo = json(&app, get(&format!("{uri}&status=Todo"), s.assignee)).await;
    assert_eq!(task_ids(&todo), vec![dated]);
}

/// A completed task is archived (MAIR-502): out of the default list, listed with `archived_at`
/// when `status` includes `Completed`.
#[actix_web::test]
#[serial]
async fn archived_tasks_are_only_listed_when_completed_is_asked() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let s = scenario(&ctx, &app).await;
    let done = patch(&s.task(), s.assignee, json!({ "status": "Completed" }));
    assert_eq!(status(&app, done).await, 204);

    let uri = format!("/api/v1/tasks/?assigned_to={}", s.assignee);
    let page = json(&app, get(&uri, s.assignee)).await;
    assert_eq!(page["total"], 0);
    let pending = json(
        &app,
        get(&format!("{uri}&status=Todo,InProgress"), s.assignee),
    )
    .await;
    assert_eq!(pending["total"], 0);

    let completed = json(&app, get(&format!("{uri}&status=Completed"), s.assignee)).await;
    assert_eq!(task_ids(&completed), vec![s.task_id]);
    assert_eq!(completed["tasks"][0]["status"], "Completed");
    assert!(completed["tasks"][0]["archived_at"].is_string());
}

/// Admin and Maire see the tasks of every project; a plain agent only those of their projects.
#[actix_web::test]
#[serial]
async fn admin_and_maire_see_the_tasks_of_every_project() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let s = scenario(&ctx, &app).await;
    let admin = ctx.user("Admin", Some("Admin")).await;
    let maire = ctx.user("Maire", Some("Maire")).await;

    let uri = format!("/api/v1/tasks/?assigned_to={}", s.assignee);
    for viewer in [admin, maire] {
        let page = json(&app, get(&uri, viewer)).await;
        assert_eq!(task_ids(&page), vec![s.task_id], "viewer {viewer}");
    }
}

/// Invalid filters are refused with a `400` naming the parameter, before any query; no token is a
/// `401`.
#[actix_web::test]
#[serial]
async fn the_tasks_list_refuses_invalid_filters_and_anonymous_callers() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let user = ctx.user("Filterer", None).await;

    for query in [
        "status=todo",
        "status=Todo,",
        "status=",
        "status=Archived",
        "assigned_to=-1",
        "assigned_to=abc",
        "due_before=tomorrow",
        "due_before=2026-13-01T00:00:00Z",
        "limit=-1",
        "offset=x",
    ] {
        let request = get(&format!("/api/v1/tasks/?{query}"), user);
        assert_eq!(status(&app, request).await, 400, "{query}");
    }
    for query in [
        "",
        "status=Todo,InProgress,Completed,Error",
        "assigned_to=1&due_before=2026-12-31T23:59:59%2B02:00&limit=0&offset=1000",
        "limit=10000",
    ] {
        let request = get(&format!("/api/v1/tasks/?{query}"), user);
        assert_eq!(status(&app, request).await, 200, "{query}");
    }
    let anonymous = TestRequest::get().uri("/api/v1/tasks/");
    assert_eq!(status(&app, anonymous).await, 401);
}
