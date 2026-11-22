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
    // A contributor and a reader are refused; the member made a manager above may change roles.
    for plain in [s.assignee, reader] {
        assert_eq!(
            status(&app, patch(&member, plain, json!({ "role": "reader" }))).await,
            403
        );
    }
    let assignee = format!("{users}{}/", s.assignee);
    assert_eq!(
        status(
            &app,
            patch(&assignee, s.member, json!({ "role": "contributor" }))
        )
        .await,
        204
    );
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

/// MAIR-1088, decision of the product owner: a member whose role is `manager` manages the
/// project like an Admin, a Maire or a Responsable; `contributor` and `reader` keep the rights of
/// a member. Every project read tells the caller `my_role` and `can_manage`.
#[actix_web::test]
async fn mair_1088_a_manager_member_manages_the_project() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let s = scenario(&ctx, &app).await;
    let users = format!("{}users/", s.project());
    let member = format!("{users}{}/", s.member);
    let assignee = format!("{users}{}/", s.assignee);

    // A plain member (contributor) reads `can_manage: false` and is refused the writes.
    let project = json(&app, get(&s.project(), s.member)).await;
    assert_eq!(project["project"]["my_role"], "contributor");
    assert_eq!(project["project"]["can_manage"], false);
    let rename = json!({ "name": "Renommé par un membre" });
    assert_eq!(
        status(&app, patch(&s.project(), s.member, rename.clone())).await,
        403
    );
    let owner_view = json(&app, get(&s.project(), s.manager)).await;
    assert_eq!(owner_view["project"]["my_role"], "manager");
    assert_eq!(owner_view["project"]["can_manage"], true);

    // Made a manager by the owner, the plain agent (role `User`) manages the project.
    let promoted = patch(&member, s.manager, json!({ "role": "manager" }));
    assert_eq!(status(&app, promoted).await, 204);
    let project = json(&app, get(&s.project(), s.member)).await;
    assert_eq!(project["project"]["my_role"], "manager");
    assert_eq!(project["project"]["can_manage"], true);
    let list = json(&app, get("/api/v1/projects/?limit=500", s.member)).await;
    let listed = list["projects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == s.project_id)
        .unwrap()
        .clone();
    assert_eq!(listed["my_role"], "manager");
    assert_eq!(listed["can_manage"], true);

    assert_eq!(
        status(&app, patch(&s.project(), s.member, rename)).await,
        204
    );
    let task = post(
        &format!("{}tasks/", s.project()),
        s.member,
        json!({ "name": "Tâche du membre gestionnaire", "fields": [] }),
    );
    assert_eq!(status(&app, task).await, 200);
    let milestone = post(
        &format!("{}milestones/", s.project()),
        s.member,
        json!({ "title": "Jalon", "due_date": "2027-01-15" }),
    );
    assert_eq!(status(&app, milestone).await, 201);
    let budget = patch(&s.project(), s.member, json!({ "budget_voted": 100 }));
    assert_eq!(status(&app, budget).await, 204);
    assert_eq!(
        status(
            &app,
            patch(&assignee, s.member, json!({ "role": "reader" }))
        )
        .await,
        204
    );
    // The owner stays manager, even for a manager member.
    let owner = format!("{users}{}/", s.manager);
    assert_eq!(
        status(&app, patch(&owner, s.member, json!({ "role": "reader" }))).await,
        409
    );
    assert_eq!(status(&app, delete(&owner, s.member)).await, 409);

    // A reader keeps the rights of a member: reads, no writes.
    let project = json(&app, get(&s.project(), s.assignee)).await;
    assert_eq!(project["project"]["my_role"], "reader");
    assert_eq!(project["project"]["can_manage"], false);
    let refused = patch(&s.project(), s.assignee, json!({ "name": "Non" }));
    assert_eq!(status(&app, refused).await, 403);
    let refused = patch(&member, s.assignee, json!({ "role": "reader" }));
    assert_eq!(status(&app, refused).await, 403);

    // Demoted, the agent loses the rights at once.
    let demoted = patch(&member, s.manager, json!({ "role": "contributor" }));
    assert_eq!(status(&app, demoted).await, 204);
    let refused = patch(&s.project(), s.member, json!({ "name": "Encore" }));
    assert_eq!(status(&app, refused).await, 403);
    assert_eq!(
        json(&app, get(&s.project(), s.member)).await["project"]["can_manage"],
        false
    );
}
