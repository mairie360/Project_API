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
    // The list carries the aggregates of the tasks and filters on them (MAIR-474).
    let urgent = json(
        &app,
        get(
            "/api/v1/projects/?priority=High&status=Active&limit=500",
            s.manager,
        ),
    )
    .await;
    let listed = urgent["projects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|project| project["id"] == s.project_id)
        .expect("the project of the urgent task is listed");
    assert_eq!(listed["priority"], "High", "an Urgent task is stored High");
    assert_eq!(listed["tasks_total"], 1);
    assert_eq!(listed["members_total"], 2);
    assert_eq!(
        urgent["summary"]["by_priority"]["high"], urgent["total"],
        "the summary counts the matching projects"
    );

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

/// Invalid filters of the list are refused with a `400` naming the parameter, before any query.
#[actix_web::test]
#[serial]
async fn the_projects_list_refuses_invalid_filters() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let user = ctx.user("Filterer", None).await;

    for query in [
        "status=active",
        "status=Active,",
        "priority=Highest",
        "priority=Urgent",
        "priority=",
        "due_before=2026-13-01T00:00:00Z",
        "due_after=tomorrow",
        "search=a%00b",
    ] {
        let request = get(&format!("/api/v1/projects/?{query}"), user);
        assert_eq!(status(&app, request).await, 400, "{query}");
    }
    let long = "a".repeat(256);
    assert_eq!(
        status(&app, get(&format!("/api/v1/projects/?search={long}"), user)).await,
        400
    );
    for query in [
        "status=Active,Suspended,Completed,Error",
        "priority=Low,Medium,High",
        "search=march%C3%A9&due_before=2026-12-31T23:59:59Z&due_after=2026-01-01T00:00:00%2B02:00",
    ] {
        let request = get(&format!("/api/v1/projects/?{query}"), user);
        assert_eq!(status(&app, request).await, 200, "{query}");
    }
}

/// One task read alone (MAIR-474): whoever sees the project sees it; a task of another project, an unknown
/// task or a project out of sight answer 404.
#[actix_web::test]
#[serial]
async fn a_task_is_read_alone_by_whoever_sees_its_project() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let s = scenario(&ctx, &app).await;

    for viewer in [s.manager, s.member, s.assignee] {
        let task = json(&app, get(&s.task(), viewer)).await;
        assert_eq!(task["id"], s.task_id, "viewer {viewer}");
        assert_eq!(task["assigned_to"], s.assignee);
        assert_eq!(task["title"], "Consulter les riverains");
    }
    for outsider in [s.outsider, s.outsider_manager] {
        assert_eq!(status(&app, get(&s.task(), outsider)).await, 404);
    }
    let unknown = format!("/api/v1/projects/{}/tasks/{}/", s.project_id, i32::MAX);
    assert_eq!(status(&app, get(&unknown, s.manager)).await, 404);

    // The same task through another project the manager sees is not found.
    let other = json(
        &app,
        post(
            "/api/v1/projects/",
            s.manager,
            serde_json::json!({ "name": "Autre projet", "description": "" }),
        ),
    )
    .await;
    let elsewhere = format!(
        "/api/v1/projects/{}/tasks/{}/",
        other["project_id"], s.task_id
    );
    assert_eq!(status(&app, get(&elsewhere, s.manager)).await, 404);
}

/// A completed task is archived (MAIR-502): it leaves the active tasks of the project detail and of
/// the task list, is counted in `tasks_archived` and listed by `…/archived-tasks/`; reopened, it
/// comes back.
#[actix_web::test]
#[serial]
async fn a_completed_task_moves_to_the_archived_tasks_and_back_when_reopened() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let s = scenario(&ctx, &app).await;
    let p = s.project();
    let ids = |page: &serde_json::Value| -> Vec<u64> {
        page["tasks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|task| task["id"].as_u64().unwrap())
            .collect()
    };

    let before = json(&app, get(&p, s.member)).await;
    assert_eq!(ids(&before), vec![s.task_id]);
    assert_eq!(before["tasks_archived"], 0);

    // The assignee completes their task.
    let done = patch(
        &s.task(),
        s.assignee,
        serde_json::json!({ "status": "Completed" }),
    );
    assert_eq!(status(&app, done).await, 204);

    let detail = json(&app, get(&p, s.member)).await;
    assert!(ids(&detail).is_empty(), "{detail}");
    assert_eq!(
        (
            detail["tasks_total"].clone(),
            detail["tasks_archived"].clone()
        ),
        (0.into(), 1.into())
    );
    let active = json(&app, get(&format!("{p}tasks/"), s.member)).await;
    assert_eq!(active["total"], 0);
    let archived = json(&app, get(&format!("{p}archived-tasks/"), s.member)).await;
    assert_eq!(ids(&archived), vec![s.task_id]);
    assert_eq!(archived["total"], 1);
    assert!(archived["tasks"][0]["archived_at"].is_string());
    // Still readable alone, with its archive date.
    let task = json(&app, get(&s.task(), s.member)).await;
    assert!(task["archived_at"].is_string());

    let reopened = patch(
        &s.task(),
        s.manager,
        serde_json::json!({ "status": "InProgress" }),
    );
    assert_eq!(status(&app, reopened).await, 204);
    let after = json(&app, get(&p, s.member)).await;
    assert_eq!(ids(&after), vec![s.task_id]);
    assert_eq!(after["tasks_archived"], 0);
    assert!(after["tasks"][0]["archived_at"].is_null());
    let none = json(&app, get(&format!("{p}archived-tasks/"), s.member)).await;
    assert_eq!(none["total"], 0);

    // The follow-up can be read from the latest comment; any other order is refused.
    let latest = json(
        &app,
        get(
            &format!("{}collaboration?comments_order=latest&limit=1", s.task()),
            s.manager,
        ),
    )
    .await;
    assert!(latest["comments"].is_array());
    assert_eq!(
        status(
            &app,
            get(
                &format!("{}collaboration?comments_order=newest", s.task()),
                s.manager
            )
        )
        .await,
        400
    );

    // Like the active list, the archived one needs the project to be visible.
    assert_eq!(
        status(&app, get(&format!("{p}archived-tasks/"), s.outsider)).await,
        404
    );
}
