//! Read cache of `GET /api/v1/projects/` (MAIR-524), against a real Redis: a second read is served
//! from the cache, every write through the API that changes the list invalidates it, and two
//! callers asking the same page never share an entry.

use actix_web::body::MessageBody;
use serde_json::{json, Value};
use serial_test::serial;

use super::{delete, get, json, patch, post, status, TestApp, TestContext};
use crate::common::fixtures::{fixture, unique_suffix};
use crate::common::fixtures_db;
use crate::init_app;

/// The projects of `user` whose name holds `tag`, read through the API.
async fn listed<B: MessageBody>(app: &impl TestApp<B>, user: u64, tag: &str) -> Value {
    json(app, get(&format!("/api/v1/projects/?search={tag}"), user)).await
}

/// Renames the project behind the API's back: no invalidation, only a read from Postgres sees it.
async fn rename_in_database(project_id: u64, title: &str) {
    fixtures_db()
        .await
        .execute(fixture(format!(
            "UPDATE projects SET title = '{title}' WHERE id = {project_id}"
        )))
        .await
        .unwrap();
}

/// A second read is served from Redis: a change made behind the API's back stays invisible until a
/// write through the API invalidates the entry.
#[actix_web::test]
#[serial]
async fn the_projects_list_is_served_from_the_cache_until_a_write() {
    let ctx = TestContext::with_redis().await;
    let app = init_app!(ctx);
    let manager = ctx.user("Manager", Some("Responsable")).await;
    let tag = format!("cache{}", unique_suffix());
    let project_id = json(
        &app,
        post(
            "/api/v1/projects/",
            manager,
            json!({ "name": format!("Marché {tag}") }),
        ),
    )
    .await["project_id"]
        .as_u64()
        .unwrap();

    let first = listed(&app, manager, &tag).await;
    assert_eq!(first["projects"][0]["name"], format!("Marché {tag}"));

    rename_in_database(project_id, &format!("Halles {tag}")).await;
    let cached = listed(&app, manager, &tag).await;
    assert_eq!(cached, first, "second read served from the cache");
    // Another query string is another entry, read from Postgres.
    let other = json(
        &app,
        get(&format!("/api/v1/projects/?search={tag}&limit=10"), manager),
    )
    .await;
    assert_eq!(other["projects"][0]["name"], format!("Halles {tag}"));

    let write = patch(
        &format!("/api/v1/projects/{project_id}/"),
        manager,
        json!({ "description": "Budget 2026" }),
    );
    assert_eq!(status(&app, write).await, 204);
    let fresh = listed(&app, manager, &tag).await;
    assert_eq!(fresh["projects"][0]["name"], format!("Halles {tag}"));
    assert_eq!(fresh["projects"][0]["description"], "Budget 2026");
}

/// After each write family (project, task, membership), the next read of a cached page shows it.
#[actix_web::test]
#[serial]
async fn every_write_shown_by_the_list_invalidates_its_cache() {
    let ctx = TestContext::with_redis().await;
    let app = init_app!(ctx);
    let manager = ctx.user("Manager", Some("Responsable")).await;
    let member = ctx.user("Member", None).await;
    let tag = format!("writes{}", unique_suffix());

    // Project creation.
    assert_eq!(listed(&app, manager, &tag).await["total"], 0);
    let created = post(
        "/api/v1/projects/",
        manager,
        json!({ "name": format!("Voirie {tag}") }),
    );
    let project_id = json(&app, created).await["project_id"].as_u64().unwrap();
    let p = format!("/api/v1/projects/{project_id}/");
    assert_eq!(
        listed(&app, manager, &tag).await["total"],
        1,
        "after create project"
    );

    // Project update.
    let renamed = patch(&p, manager, json!({ "name": format!("Chaussée {tag}") }));
    assert_eq!(status(&app, renamed).await, 204);
    assert_eq!(
        listed(&app, manager, &tag).await["projects"][0]["name"],
        format!("Chaussée {tag}"),
        "after update project"
    );

    // Member added, then a task (aggregated in the list), its patch and its deletion.
    let added = post(&format!("{p}users/"), manager, json!({ "user_id": member }));
    assert_eq!(status(&app, added).await, 200);
    assert_eq!(
        listed(&app, manager, &tag).await["projects"][0]["members_total"],
        1,
        "after add member"
    );

    let task = post(
        &format!("{p}tasks/"),
        manager,
        json!({ "name": "Marquage", "fields": [], "assigned_to": member }),
    );
    let task_id = json(&app, task).await["task_id"].as_u64().unwrap();
    let t = format!("{p}tasks/{task_id}/");
    assert_eq!(
        listed(&app, manager, &tag).await["projects"][0]["tasks_total"],
        1,
        "after create task"
    );

    let urgent = patch(
        &t,
        manager,
        json!({ "priority": "High", "due_date": "2026-11-02T00:00:00Z" }),
    );
    assert_eq!(status(&app, urgent).await, 204);
    let after_patch = listed(&app, manager, &tag).await;
    // The priority of the project is its own (MAIR-558): a task does not change it.
    assert_eq!(
        after_patch["projects"][0]["priority"], "Medium",
        "after patch task"
    );
    assert_eq!(
        after_patch["projects"][0]["due_date"],
        "2026-11-02T00:00:00Z"
    );

    // Removing the member also unassigns their task, in the same write.
    let removed = delete(&format!("{p}users/{member}/"), manager);
    assert_eq!(status(&app, removed).await, 204);
    assert_eq!(
        listed(&app, manager, &tag).await["projects"][0]["members_total"],
        0,
        "after remove member"
    );

    assert_eq!(status(&app, delete(&t, manager)).await, 204);
    assert_eq!(
        listed(&app, manager, &tag).await["projects"][0]["tasks_total"],
        0,
        "after delete task"
    );

    // Project closure, then deletion.
    let closed = patch(&format!("{p}close"), manager, json!({}));
    assert_eq!(status(&app, closed).await, 200);
    assert_eq!(
        listed(&app, manager, &tag).await["projects"][0]["status"],
        "Completed",
        "after close project"
    );
    assert_eq!(status(&app, delete(&p, manager)).await, 204);
    assert_eq!(
        listed(&app, manager, &tag).await["total"],
        0,
        "after delete project"
    );
}

/// Two callers asking the same query string get their own entry: the first one's page is never
/// served to the second, whichever reads first.
#[actix_web::test]
#[serial]
async fn two_callers_with_the_same_query_never_share_an_entry() {
    let ctx = TestContext::with_redis().await;
    let app = init_app!(ctx);
    let owner = ctx.user("Owner", Some("Responsable")).await;
    let outsider = ctx.user("Outsider", None).await;
    let other_owner = ctx.user("OtherOwner", Some("Responsable")).await;
    let tag = format!("isolation{}", unique_suffix());
    for (user, name) in [(owner, "Écoles"), (other_owner, "Piscine")] {
        let created = post(
            "/api/v1/projects/",
            user,
            json!({ "name": format!("{name} {tag}") }),
        );
        assert_eq!(status(&app, created).await, 200);
    }

    // The owner reads first: their page must not leak to the outsider.
    let mine = listed(&app, owner, &tag).await;
    assert_eq!(mine["total"], 1);
    assert_eq!(mine["projects"][0]["name"], format!("Écoles {tag}"));
    assert_eq!(listed(&app, outsider, &tag).await["total"], 0);
    // The other owner sees only their own project, whatever was cached before.
    let theirs = listed(&app, other_owner, &tag).await;
    assert_eq!(theirs["total"], 1);
    assert_eq!(theirs["projects"][0]["name"], format!("Piscine {tag}"));
    // And the cached entries stay per caller.
    assert_eq!(listed(&app, owner, &tag).await, mine);
    assert_eq!(listed(&app, outsider, &tag).await["total"], 0);
}
