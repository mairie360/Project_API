//! `group_id` was accepted by `POST /api/v1/projects/` and documented as giving the group's
//! members access, but ignored: they never saw the project. A project is now linked to a group of
//! its creator (any group for Admin and Maire), at creation or later, and every current and future
//! member of that group sees it.

use serde_json::{json, Value};

use super::{get, json, patch, post, status, TestContext};
use crate::common::fixtures::create_group;
use crate::init_app;

/// Ids of the projects of a `GET /api/v1/projects/` answer.
fn project_ids(list: &Value) -> Vec<u64> {
    list["projects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|project| project["id"].as_u64().unwrap())
        .collect()
}

#[actix_web::test]
async fn mair_683_group_members_see_a_project_created_with_their_group() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let manager = ctx.user("GroupManager", Some("Responsable")).await;
    let agent = ctx.user("GroupAgent", Some("User")).await;
    let group = create_group(manager, &[manager, agent]).await;

    let offered = json(&app, get("/api/v1/project-groups/", manager)).await;
    assert!(offered["groups"]
        .as_array()
        .unwrap()
        .iter()
        .any(|g| g["id"] == group));

    let created = json(
        &app,
        post(
            "/api/v1/projects/",
            manager,
            json!({ "name": "Voirie 2026", "group_id": group }),
        ),
    )
    .await;
    let project = created["project_id"].as_u64().unwrap();

    // The bug: the agent of the group did not see it.
    let list = json(&app, get("/api/v1/projects/", agent)).await;
    assert!(project_ids(&list).contains(&project));
    let detail = json(&app, get(&format!("/api/v1/projects/{project}/"), agent)).await;
    assert_eq!(detail["project"]["group"]["id"], group);

    // A member who joins the group later sees it too.
    let newcomer = ctx.user("GroupNewcomer", Some("User")).await;
    assert_eq!(
        status(&app, get(&format!("/api/v1/projects/{project}/"), newcomer)).await,
        404
    );
    create_group_member(group, newcomer).await;
    assert_eq!(
        status(&app, get(&format!("/api/v1/projects/{project}/"), newcomer)).await,
        200
    );
    // Seeing the project does not give the right to manage it.
    assert_eq!(
        status(
            &app,
            patch(
                &format!("/api/v1/projects/{project}/"),
                agent,
                json!({ "name": "Renamed" })
            )
        )
        .await,
        403
    );
}

#[actix_web::test]
async fn mair_683_unknown_or_foreign_group_is_refused_and_nothing_is_created() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let manager = ctx.user("ForeignManager", Some("Responsable")).await;
    let stranger = ctx.user("ForeignOwner", Some("Responsable")).await;
    let foreign = create_group(stranger, &[stranger]).await;

    let before = json(&app, get("/api/v1/projects/", manager)).await["total"].clone();
    for group in [foreign, 2_000_000] {
        assert_eq!(
            status(
                &app,
                post(
                    "/api/v1/projects/",
                    manager,
                    json!({ "name": "Refused", "group_id": group })
                )
            )
            .await,
            404
        );
    }
    let after = json(&app, get("/api/v1/projects/", manager)).await["total"].clone();
    assert_eq!(before, after, "no project was created");
    let offered = json(&app, get("/api/v1/project-groups/", manager)).await;
    assert!(!offered["groups"]
        .as_array()
        .unwrap()
        .iter()
        .any(|g| g["id"] == foreign));

    // The Admin may link any group.
    let admin = json(
        &app,
        post(
            "/api/v1/projects/",
            1,
            json!({ "name": "Linked by the Admin", "group_id": foreign }),
        ),
    )
    .await;
    let project = admin["project_id"].as_u64().unwrap();
    assert_eq!(
        status(&app, get(&format!("/api/v1/projects/{project}/"), stranger)).await,
        200
    );
}

#[actix_web::test]
async fn mair_683_changing_or_removing_the_group_moves_the_visibility() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let manager = ctx.user("SwitchManager", Some("Responsable")).await;
    let roads = ctx.user("SwitchRoads", Some("User")).await;
    let parks = ctx.user("SwitchParks", Some("User")).await;
    let roads_group = create_group(manager, &[manager, roads]).await;
    let parks_group = create_group(manager, &[manager, parks]).await;
    let stranger = ctx.user("SwitchStranger", Some("Responsable")).await;
    let foreign = create_group(stranger, &[stranger]).await;

    let created = json(
        &app,
        post(
            "/api/v1/projects/",
            manager,
            json!({ "name": "Square", "group_id": roads_group }),
        ),
    )
    .await;
    let uri = format!(
        "/api/v1/projects/{}/",
        created["project_id"].as_u64().unwrap()
    );

    // Absent `group_id`: the group is kept.
    assert_eq!(
        status(&app, patch(&uri, manager, json!({ "name": "Square 2" }))).await,
        204
    );
    assert_eq!(status(&app, get(&uri, roads)).await, 200);

    // A group the manager may not link: 404 and nothing changes, the name neither.
    assert_eq!(
        status(
            &app,
            patch(
                &uri,
                manager,
                json!({ "name": "Square 3", "group_id": foreign })
            )
        )
        .await,
        404
    );
    let detail = json(&app, get(&uri, manager)).await;
    assert_eq!(detail["project"]["name"], "Square 2");
    assert_eq!(detail["project"]["group"]["id"], roads_group);

    // Replaced: the former group loses it, the new one sees it.
    assert_eq!(
        status(
            &app,
            patch(&uri, manager, json!({ "group_id": parks_group }))
        )
        .await,
        204
    );
    assert_eq!(status(&app, get(&uri, roads)).await, 404);
    assert_eq!(status(&app, get(&uri, parks)).await, 200);

    // `null` removes it.
    assert_eq!(
        status(&app, patch(&uri, manager, json!({ "group_id": null }))).await,
        204
    );
    assert_eq!(status(&app, get(&uri, parks)).await, 404);
    let detail = json(&app, get(&uri, manager)).await;
    assert!(detail["project"]["group"].is_null());
}

async fn create_group_member(group: u64, user: u64) {
    use crate::common::fixtures::fixture;
    crate::common::fixtures_db()
        .await
        .execute(fixture(format!(
            "INSERT INTO group_members (group_id, user_id) VALUES ({group}, {user})"
        )))
        .await
        .unwrap();
}
