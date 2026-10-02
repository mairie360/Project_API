//! Refusals: whoever is not allowed to see or change a project, its tasks or its members gets
//! `401`, `403` or `404`, and nothing is changed.

use actix_web::test::TestRequest;
use serde_json::json;
use serial_test::serial;

use actix_web::body::MessageBody;

use super::{delete, get, json, jwt_for, patch, post, status, status_raw, TestApp, TestContext};
use crate::init_app;

/// A project owned by a Responsable, with a plain agent as member and a task assigned to them.
pub struct Scenario {
    pub manager: u64,
    pub member: u64,
    pub assignee: u64,
    pub outsider: u64,
    pub outsider_manager: u64,
    pub project_id: u64,
    pub task_id: u64,
}

impl Scenario {
    pub fn project(&self) -> String {
        format!("/api/v1/projects/{}/", self.project_id)
    }

    pub fn task(&self) -> String {
        format!(
            "/api/v1/projects/{}/tasks/{}/",
            self.project_id, self.task_id
        )
    }
}

pub async fn scenario<B: MessageBody>(ctx: &TestContext, app: &impl TestApp<B>) -> Scenario {
    let manager = ctx.user("Manager", Some("Responsable")).await;
    let member = ctx.user("Member", None).await;
    let assignee = ctx.user("Assignee", None).await;
    let outsider = ctx.user("Outsider", None).await;
    // A Responsable who shares no group with the project's people does not see it.
    let outsider_manager = ctx.user("OtherManager", Some("Responsable")).await;

    let created = json(
        app,
        post(
            "/api/v1/projects/",
            manager,
            json!({ "name": "Réfection de la place du marché" }),
        ),
    )
    .await;
    let project_id = created["project_id"].as_u64().unwrap();
    for user in [member, assignee] {
        let status = status(
            app,
            post(
                &format!("/api/v1/projects/{project_id}/users/"),
                manager,
                json!({ "user_id": user }),
            ),
        )
        .await;
        assert_eq!(status, 200);
    }
    let task = json(
        app,
        post(
            &format!("/api/v1/projects/{project_id}/tasks/"),
            manager,
            json!({ "name": "Consulter les riverains", "fields": [], "assigned_to": assignee }),
        ),
    )
    .await;
    Scenario {
        manager,
        member,
        assignee,
        outsider,
        outsider_manager,
        project_id,
        task_id: task["task_id"].as_u64().unwrap(),
    }
}

#[actix_web::test]
#[serial]
async fn requests_without_a_valid_jwt_are_rejected() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);

    let anonymous = TestRequest::get().uri("/api/v1/projects/");
    assert_eq!(status(&app, anonymous).await, 401);

    let forged = TestRequest::get()
        .uri("/api/v1/projects/")
        .insert_header(("Authorization", "Bearer eyJhbGciOiJIUzI1NiJ9.e30.forged"));
    assert_eq!(status(&app, forged).await, 401);

    // A well-signed token for an account that does not exist: the lib's middleware answers 404.
    let ghost = TestRequest::get()
        .uri("/api/v1/projects/")
        .insert_header(("Authorization", jwt_for(i32::MAX as u64)));
    assert_eq!(status(&app, ghost).await, 404);
}

#[actix_web::test]
#[serial]
async fn only_managers_can_create_a_project() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let agent = ctx.user("Agent", None).await;

    let status = status(
        &app,
        post(
            "/api/v1/projects/",
            agent,
            json!({ "name": "Projet sans droit" }),
        ),
    )
    .await;
    assert_eq!(status, 403);
}

#[actix_web::test]
#[serial]
async fn an_outsider_can_neither_see_nor_change_the_project() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let s = scenario(&ctx, &app).await;

    for outsider in [s.outsider, s.outsider_manager] {
        let p = s.project();
        let t = s.task();
        let requests = [
            get(&p, outsider),
            patch(&p, outsider, json!({ "status": "Suspended" })),
            delete(&p, outsider),
            patch(&format!("{p}close"), outsider, json!({})),
            get(&format!("{p}tasks/"), outsider),
            post(
                &format!("{p}tasks/"),
                outsider,
                json!({ "name": "Intrusion", "fields": [] }),
            ),
            patch(&t, outsider, json!({ "status": "Completed" })),
            delete(&t, outsider),
            get(&format!("{t}collaboration"), outsider),
            post(
                &format!("{t}comments"),
                outsider,
                json!({ "message": "Bonjour" }),
            ),
            get(&format!("{p}users/"), outsider),
            post(
                &format!("{p}users/"),
                outsider,
                json!({ "user_id": outsider }),
            ),
            delete(&format!("{p}users/{}/", s.member), outsider),
        ];
        for request in requests {
            let req = request.to_request();
            let label = format!("{} {}", req.method(), req.path());
            let status = status_raw(&app, req).await;
            assert_eq!(status, 404, "{label} by user {outsider}");
        }

        let list = json(&app, get("/api/v1/projects/", outsider)).await;
        let ids: Vec<u64> = list["projects"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p["id"].as_u64().unwrap())
            .collect();
        assert!(!ids.contains(&s.project_id));
    }

    // Nothing was changed by the refused requests.
    let project = json(&app, get(&s.project(), s.manager)).await;
    assert_eq!(project["project"]["status"], "Active");
    assert_eq!(project["tasks_total"], 1);
}

#[actix_web::test]
#[serial]
async fn a_member_without_manager_role_can_only_read() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let s = scenario(&ctx, &app).await;
    let p = s.project();
    let t = s.task();

    assert_eq!(status(&app, get(&p, s.member)).await, 200);
    assert_eq!(
        status(&app, get(&format!("{p}tasks/"), s.member)).await,
        200
    );
    assert_eq!(
        status(&app, get(&format!("{p}users/"), s.member)).await,
        200
    );

    let forbidden = [
        patch(&p, s.member, json!({ "name": "Renommé" })),
        patch(&format!("{p}close"), s.member, json!({})),
        delete(&p, s.member),
        post(
            &format!("{p}tasks/"),
            s.member,
            json!({ "name": "Nouvelle tâche", "fields": [] }),
        ),
        // Not assigned to the task: no collaboration either.
        patch(&t, s.member, json!({ "status": "Completed" })),
        get(&format!("{t}collaboration"), s.member),
        post(
            &format!("{t}comments"),
            s.member,
            json!({ "message": "Bonjour" }),
        ),
        delete(&t, s.member),
        post(
            &format!("{p}users/"),
            s.member,
            json!({ "user_id": s.outsider }),
        ),
        delete(&format!("{p}users/{}/", s.assignee), s.member),
    ];
    for request in forbidden {
        let req = request.to_request();
        let label = format!("{} {}", req.method(), req.path());
        let status = status_raw(&app, req).await;
        assert_eq!(status, 403, "{label}");
    }
}

#[actix_web::test]
#[serial]
async fn the_assignee_may_only_change_the_status_of_their_task() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let s = scenario(&ctx, &app).await;
    let t = s.task();

    let status_only = patch(&t, s.assignee, json!({ "status": "InProgress" }));
    assert_eq!(status(&app, status_only).await, 204);
    let rename = patch(&t, s.assignee, json!({ "name": "Renommée par l'agent" }));
    assert_eq!(status(&app, rename).await, 403);
    let reassign = patch(&t, s.assignee, json!({ "assigned_to": null }));
    assert_eq!(status(&app, reassign).await, 403);
    assert_eq!(status(&app, delete(&t, s.assignee)).await, 403);

    let comment = post(
        &format!("{t}comments"),
        s.assignee,
        json!({ "message": "Réunion calée." }),
    );
    assert_eq!(status(&app, comment).await, 201);
    let collaboration = json(&app, get(&format!("{t}collaboration"), s.assignee)).await;
    assert_eq!(collaboration["comments_total"], 1);
}

#[actix_web::test]
#[serial]
async fn a_task_is_only_reachable_through_its_own_project() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let s = scenario(&ctx, &app).await;

    // The manager owns a second project: the task id of the first one must not be reachable
    // through the URL of the second one.
    let other = json(
        &app,
        post(
            "/api/v1/projects/",
            s.manager,
            json!({ "name": "Autre projet" }),
        ),
    )
    .await;
    let other_id = other["project_id"].as_u64().unwrap();
    let crossed = format!("/api/v1/projects/{other_id}/tasks/{}/", s.task_id);

    assert_eq!(
        status(
            &app,
            patch(&crossed, s.manager, json!({ "status": "Completed" }))
        )
        .await,
        404
    );
    assert_eq!(status(&app, delete(&crossed, s.manager)).await, 404);
    assert_eq!(
        status(&app, get(&format!("{crossed}collaboration"), s.manager)).await,
        404
    );
    assert_eq!(
        status(
            &app,
            post(
                &format!("{crossed}comments"),
                s.manager,
                json!({ "message": "x" })
            )
        )
        .await,
        404
    );
    // The task is still in its project.
    let project = json(&app, get(&s.project(), s.manager)).await;
    assert_eq!(project["tasks_total"], 1);
}

#[actix_web::test]
#[serial]
async fn a_task_cannot_be_assigned_outside_the_project() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let s = scenario(&ctx, &app).await;

    let reassign = patch(&s.task(), s.manager, json!({ "assigned_to": s.outsider }));
    assert_eq!(status(&app, reassign).await, 400);
    let create = post(
        &format!("{}tasks/", s.project()),
        s.manager,
        json!({ "name": "Tâche", "fields": [], "assigned_to": s.outsider }),
    );
    assert_eq!(status(&app, create).await, 400);
}

#[actix_web::test]
#[serial]
async fn invalid_path_segments_and_bodies_are_rejected() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let s = scenario(&ctx, &app).await;

    assert_eq!(
        status(&app, get("/api/v1/projects/abc/", s.manager)).await,
        400
    );
    assert_eq!(
        status(&app, get("/api/v1/projects/-1/", s.manager)).await,
        400,
        "a negative id is not an id"
    );
    let task = format!("{}tasks/x/", s.project());
    assert_eq!(
        status(
            &app,
            patch(&task, s.manager, json!({ "status": "Completed" }))
        )
        .await,
        400
    );
    let blank = patch(&s.project(), s.manager, json!({ "name": "   " }));
    assert_eq!(status(&app, blank).await, 400);
    let nul = post(
        "/api/v1/projects/",
        s.manager,
        json!({ "name": "Place\u{0}du marché" }),
    );
    assert_eq!(status(&app, nul).await, 400);
    let missing = post(&format!("{}users/", s.project()), s.manager, json!({}));
    assert_eq!(status(&app, missing).await, 400);
}

#[actix_web::test]
#[serial]
async fn ids_beyond_int4_do_not_alias_existing_rows() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let s = scenario(&ctx, &app).await;
    // `as i32` used to wrap 2^32 + n to n: these ids designated the scenario's rows (MAIR-422).
    let wrap = 1_u64 << 32;
    let aliased_project = format!("/api/v1/projects/{}/", wrap + s.project_id);
    assert_eq!(status(&app, get(&aliased_project, s.manager)).await, 404);
    assert_eq!(status(&app, delete(&aliased_project, s.manager)).await, 404);
    let aliased_task = format!("{}tasks/{}/", s.project(), wrap + s.task_id);
    assert_eq!(
        status(
            &app,
            patch(&aliased_task, s.manager, json!({ "status": "Completed" }))
        )
        .await,
        404
    );

    let aliased_assignee = patch(
        &s.task(),
        s.manager,
        json!({ "assigned_to": wrap + s.member }),
    );
    assert_eq!(status(&app, aliased_assignee).await, 400);
    let aliased_user = post(
        &format!("{}users/", s.project()),
        s.manager,
        json!({ "user_id": wrap + s.outsider }),
    );
    assert_eq!(status(&app, aliased_user).await, 404);

    // Nothing was changed through the aliases.
    let project = json(&app, get(&s.project(), s.manager)).await;
    assert_eq!(project["tasks"][0]["assigned_to"], s.assignee);
    assert_eq!(project["users"].as_array().unwrap().len(), 2);
}
