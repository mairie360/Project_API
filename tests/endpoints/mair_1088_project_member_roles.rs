//! MAIR-1088: every member read carries the member's `role` in the project (`manager`,
//! `contributor`, `reader`; the owner reads `manager`). The managers choose it at the addition
//! (default `contributor`) and change it with `PATCH …/users/{user_id}/`; the owner can neither be
//! changed nor removed (`409`).

use serde_json::{json, Value};

use super::access::scenario;
use super::{delete, get, json, patch, post, status, TestContext};
use crate::init_app;

fn role(page: &Value, list: &str, user: u64) -> Value {
    page[list]
        .as_array()
        .unwrap()
        .iter()
        .find(|member| member["id"] == user)
        .map(|member| member["role"].clone())
        .unwrap_or(Value::Null)
}

#[actix_web::test]
async fn mair_1088_managers_give_roles_the_owner_stays_manager() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let s = scenario(&ctx, &app).await;
    let users = format!("{}users/", s.project());

    let page = json(&app, get(&users, s.member)).await;
    assert_eq!(role(&page, "users", s.member), "contributor", "{page}");
    assert_eq!(role(&page, "users", s.assignee), "contributor");

    // Added with a role, or the owner added as a member: the owner reads manager.
    let reader = ctx.user("Lecteur", None).await;
    let added = post(
        &users,
        s.manager,
        json!({ "user_id": reader, "role": "reader" }),
    );
    assert_eq!(status(&app, added).await, 200);
    let owner = post(
        &users,
        s.manager,
        json!({ "user_id": s.manager, "role": "reader" }),
    );
    assert_eq!(status(&app, owner).await, 200);
    let refused = post(
        &users,
        s.manager,
        json!({ "user_id": s.outsider, "role": "admin" }),
    );
    assert_eq!(status(&app, refused).await, 400);
    let page = json(&app, get(&users, s.member)).await;
    assert_eq!(role(&page, "users", reader), "reader");
    assert_eq!(role(&page, "users", s.manager), "manager");
    assert_eq!(role(&page, "users", s.outsider), Value::Null);
    let project = json(&app, get(&s.project(), s.member)).await;
    assert_eq!(role(&project, "users", reader), "reader");
    assert_eq!(role(&project, "users", s.manager), "manager");
    let list = json(&app, get("/api/v1/projects/?limit=500", s.member)).await;
    let listed = list["projects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == s.project_id)
        .unwrap()
        .clone();
    assert_eq!(role(&listed, "members", reader), "reader", "{listed}");

    // A manager changes a role.
    let member = format!("{users}{}/", s.member);
    assert_eq!(
        status(
            &app,
            patch(&member, s.manager, json!({ "role": "manager" }))
        )
        .await,
        204
    );
    let page = json(&app, get(&users, s.member)).await;
    assert_eq!(role(&page, "users", s.member), "manager");

    // Refusals: bad body, not a member, the owner, not a manager, out of sight.
    for body in [
        json!({}),
        json!({ "role": "Manager" }),
        json!({ "role": null }),
    ] {
        assert_eq!(
            status(&app, patch(&member, s.manager, body.clone())).await,
            400,
            "{body}"
        );
    }
    let stranger = format!("{users}{}/", s.outsider);
    assert_eq!(
        status(
            &app,
            patch(&stranger, s.manager, json!({ "role": "reader" }))
        )
        .await,
        404
    );
    let owner = format!("{users}{}/", s.manager);
    assert_eq!(
        status(&app, patch(&owner, s.manager, json!({ "role": "reader" }))).await,
        409
    );
    for plain in [s.member, s.assignee] {
        assert_eq!(
            status(&app, patch(&member, plain, json!({ "role": "reader" }))).await,
            403
        );
    }
    assert_eq!(
        status(
            &app,
            patch(&member, s.outsider, json!({ "role": "reader" }))
        )
        .await,
        404
    );

    // The owner cannot be removed, a member can.
    assert_eq!(status(&app, delete(&owner, s.manager)).await, 409);
    let page = json(&app, get(&users, s.manager)).await;
    assert_eq!(role(&page, "users", s.manager), "manager");
    let reader_path = format!("{users}{reader}/");
    assert_eq!(status(&app, delete(&reader_path, s.manager)).await, 204);
    assert_eq!(
        status(
            &app,
            patch(&reader_path, s.manager, json!({ "role": "manager" }))
        )
        .await,
        404
    );
}
