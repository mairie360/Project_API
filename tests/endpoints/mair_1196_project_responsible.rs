//! MAIR-1196: a project records its responsible (`projects.responsible_id`). `responsible_id` on
//! `POST /projects/` and `PATCH /projects/{id}/` sets it and makes that user a `manager` member; the
//! detail and the list read it back as `responsible`, whatever the order of the members; removing
//! the responsible from the members clears it.

use actix_web::body::MessageBody;
use serde_json::{json, Value};

use super::{delete, get, json, patch, post, status, TestApp, TestContext};
use crate::init_app;

/// `responsible` of the project in its detail and in the list, as read by `user`.
async fn responsibles<B: MessageBody>(
    app: &impl TestApp<B>,
    project_id: u64,
    user: u64,
) -> (Value, Value) {
    let detail = json(app, get(&format!("/api/v1/projects/{project_id}/"), user)).await;
    let list = json(app, get("/api/v1/projects/?limit=500", user)).await;
    let listed = list["projects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|project| project["id"] == project_id)
        .unwrap_or_else(|| panic!("project {project_id} not listed: {list}"))
        .clone();
    (
        detail["project"]["responsible"].clone(),
        listed["responsible"].clone(),
    )
}

/// Role of `user` among the members of the project, `None` when not a member.
async fn member_role<B: MessageBody>(
    app: &impl TestApp<B>,
    project_id: u64,
    owner: u64,
    user: u64,
) -> Option<String> {
    let members = json(
        app,
        get(&format!("/api/v1/projects/{project_id}/users/"), owner),
    )
    .await;
    members["users"]
        .as_array()
        .unwrap()
        .iter()
        .find(|member| member["id"] == user)
        .map(|member| member["role"].as_str().unwrap().to_string())
}

#[actix_web::test]
async fn mair_1196_the_chosen_responsible_stays_whatever_the_members() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    // Names sort "Aaron" < "Bruno" < "Zoe": the responsible is the last member by name.
    let owner = ctx.user("Aaron", Some("Responsable")).await;
    let other = ctx.user("Bruno", None).await;
    let responsible = ctx.user("Zoe", None).await;

    let created = json(
        &app,
        post(
            "/api/v1/projects/",
            owner,
            json!({ "name": "Place du marché", "responsible_id": responsible }),
        ),
    )
    .await;
    let project_id = created["project_id"].as_u64().unwrap();
    let expected = json!({ "id": responsible, "name": "Zoe Test" });
    assert_eq!(
        responsibles(&app, project_id, owner).await,
        (expected.clone(), expected.clone())
    );
    assert_eq!(
        member_role(&app, project_id, owner, responsible)
            .await
            .as_deref(),
        Some("manager")
    );

    // Add a contributor who sorts first, remove them, add them back; then the owner as a member
    // (a `manager` sorting first): the responsible never changes.
    let users = format!("/api/v1/projects/{project_id}/users/");
    let add = post(&users, owner, json!({ "user_id": other }));
    assert_eq!(status(&app, add).await, 200);
    assert_eq!(
        responsibles(&app, project_id, owner).await,
        (expected.clone(), expected.clone())
    );
    let remove = delete(&format!("{users}{other}/"), owner);
    assert_eq!(status(&app, remove).await, 204);
    let add_back = post(&users, owner, json!({ "user_id": other }));
    assert_eq!(status(&app, add_back).await, 200);
    let add_owner = post(&users, owner, json!({ "user_id": owner }));
    assert_eq!(status(&app, add_owner).await, 200);
    assert_eq!(
        responsibles(&app, project_id, responsible).await,
        (expected.clone(), expected)
    );
}

#[actix_web::test]
async fn mair_1196_a_patch_replaces_or_clears_the_responsible_and_promotes_them() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let owner = ctx.user("Aaron", Some("Responsable")).await;
    let member = ctx.user("Bruno", None).await;

    // A project created without a responsible has none.
    let created = json(
        &app,
        post("/api/v1/projects/", owner, json!({ "name": "Voirie" })),
    )
    .await;
    let project_id = created["project_id"].as_u64().unwrap();
    let p = format!("/api/v1/projects/{project_id}/");
    assert_eq!(
        responsibles(&app, project_id, owner).await,
        (Value::Null, Value::Null)
    );
    let add = post(
        &format!("{p}users/"),
        owner,
        json!({ "user_id": member, "role": "reader" }),
    );
    assert_eq!(status(&app, add).await, 200);

    // A plain member cannot choose the responsible.
    let refused = patch(&p, member, json!({ "responsible_id": member }));
    assert_eq!(status(&app, refused).await, 403);

    // The manager makes the reader responsible: they are promoted to `manager`.
    let set = patch(&p, owner, json!({ "responsible_id": member }));
    assert_eq!(status(&app, set).await, 204);
    let expected = json!({ "id": member, "name": "Bruno Test" });
    assert_eq!(
        responsibles(&app, project_id, owner).await,
        (expected.clone(), expected)
    );
    assert_eq!(
        member_role(&app, project_id, owner, member)
            .await
            .as_deref(),
        Some("manager")
    );

    // Absent keeps it; `null` clears it, the member keeps their role.
    let renamed = patch(&p, owner, json!({ "name": "Voirie 2027" }));
    assert_eq!(status(&app, renamed).await, 204);
    assert_eq!(responsibles(&app, project_id, owner).await.0["id"], member);
    let cleared = patch(&p, owner, json!({ "responsible_id": null }));
    assert_eq!(status(&app, cleared).await, 204);
    assert_eq!(
        responsibles(&app, project_id, owner).await,
        (Value::Null, Value::Null)
    );
    assert_eq!(
        member_role(&app, project_id, owner, member)
            .await
            .as_deref(),
        Some("manager")
    );
}

#[actix_web::test]
async fn mair_1196_removing_the_responsible_from_the_members_clears_it() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let owner = ctx.user("Aaron", Some("Responsable")).await;
    let responsible = ctx.user("Zoe", None).await;
    let created = json(
        &app,
        post(
            "/api/v1/projects/",
            owner,
            json!({ "name": "Cantine", "responsible_id": responsible }),
        ),
    )
    .await;
    let project_id = created["project_id"].as_u64().unwrap();

    let remove = delete(
        &format!("/api/v1/projects/{project_id}/users/{responsible}/"),
        owner,
    );
    assert_eq!(status(&app, remove).await, 204);
    assert_eq!(
        responsibles(&app, project_id, owner).await,
        (Value::Null, Value::Null)
    );
    assert_eq!(
        member_role(&app, project_id, owner, responsible).await,
        None
    );
}

#[actix_web::test]
async fn mair_1196_an_unknown_responsible_is_refused_and_changes_nothing() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let owner = ctx.user("Aaron", Some("Responsable")).await;
    let unknown = u64::from(i32::MAX.unsigned_abs());

    let refused = post(
        "/api/v1/projects/",
        owner,
        json!({ "name": "Fantôme", "responsible_id": unknown }),
    );
    assert_eq!(status(&app, refused).await, 400);
    let list = json(&app, get("/api/v1/projects/?search=Fant", owner)).await;
    assert_eq!(list["total"], 0, "{list}");

    let created = json(
        &app,
        post("/api/v1/projects/", owner, json!({ "name": "Parc" })),
    )
    .await;
    let project_id = created["project_id"].as_u64().unwrap();
    let p = format!("/api/v1/projects/{project_id}/");
    let refused = patch(
        &p,
        owner,
        json!({ "responsible_id": unknown, "name": "Changé" }),
    );
    assert_eq!(status(&app, refused).await, 400);
    let project = json(&app, get(&p, owner)).await;
    assert_eq!(project["project"]["name"], "Parc");
    assert_eq!(project["project"]["responsible"], Value::Null);
}
