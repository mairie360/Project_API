//! A closed project (`Completed`) is read-only. Every write on it, its tasks, members,
//! milestones, statuses and comments answers `409 Conflict` `Project is closed.` and changes
//! nothing; reads, deletion and closing again still work. Its managers reopen it with a `PATCH`
//! holding only a status other than `Completed`, which gives every write back. A caller without
//! the rights keeps their `403` / `404`, and a suspended project is not closed.

use actix_web::body::MessageBody;
use actix_web::test::TestRequest;
use serde_json::json;

use super::access::{scenario, Scenario};
use super::{delete, get, json, patch, post, status, TestApp, TestContext};
use crate::init_app;

/// Status and body of a request.
async fn answer<B: MessageBody>(app: &impl TestApp<B>, request: TestRequest) -> (u16, String) {
    let response = actix_web::test::call_service(app, request.to_request()).await;
    let code = response.status().as_u16();
    let body = actix_web::test::read_body(response).await;
    (code, String::from_utf8_lossy(&body).into_owned())
}

/// Closes the scenario's project with `PATCH …/close`.
async fn close<B: MessageBody>(app: &impl TestApp<B>, s: &Scenario) {
    let closed = patch(&format!("{}close", s.project()), s.manager, json!({}));
    assert_eq!(status(app, closed).await, 200);
}

/// One request of every write family on the scenario's project, sent by `user`.
fn writes(
    s: &Scenario,
    user: u64,
    milestone: u64,
    status_id: u64,
) -> Vec<(&'static str, TestRequest)> {
    let p = s.project();
    let t = s.task();
    vec![
        (
            "update the project",
            patch(&p, user, json!({ "name": "Renommé" })),
        ),
        (
            "reopen and rename at once",
            patch(&p, user, json!({ "status": "Active", "name": "Renommé" })),
        ),
        (
            "set Completed again by PATCH",
            patch(&p, user, json!({ "status": "Completed" })),
        ),
        (
            "create a task",
            post(
                &format!("{p}tasks/"),
                user,
                json!({ "name": "Nouvelle tâche", "fields": [] }),
            ),
        ),
        (
            "update a task",
            patch(&t, user, json!({ "name": "Tâche renommée" })),
        ),
        (
            "move a task",
            patch(&t, user, json!({ "status": "InProgress" })),
        ),
        (
            "comment a task",
            post(&format!("{t}comments"), user, json!({ "message": "Vu." })),
        ),
        ("delete a task", delete(&t, user)),
        (
            "comment the project",
            post(&format!("{p}comments/"), user, json!({ "message": "Vu." })),
        ),
        (
            "add a member",
            post(
                &format!("{p}users/"),
                user,
                json!({ "user_id": s.outsider }),
            ),
        ),
        (
            "change a member's role",
            patch(
                &format!("{p}users/{}/", s.member),
                user,
                json!({ "role": "reader" }),
            ),
        ),
        (
            "remove a member",
            delete(&format!("{p}users/{}/", s.member), user),
        ),
        (
            "add a milestone",
            post(
                &format!("{p}milestones/"),
                user,
                json!({ "title": "Réception", "due_date": "2027-06-30" }),
            ),
        ),
        (
            "update a milestone",
            patch(
                &format!("{p}milestones/{milestone}/"),
                user,
                json!({ "reached": true }),
            ),
        ),
        (
            "delete a milestone",
            delete(&format!("{p}milestones/{milestone}/"), user),
        ),
        (
            "add a status",
            post(
                &format!("{p}statuses/"),
                user,
                json!({ "name": "À valider", "color": "#D97706", "category": "InProgress", "position": 2 }),
            ),
        ),
        (
            "update a status",
            patch(
                &format!("{p}statuses/{status_id}/"),
                user,
                json!({ "name": "Renommé" }),
            ),
        ),
        (
            "delete a status",
            delete(&format!("{p}statuses/{status_id}/"), user),
        ),
    ]
}

/// A milestone and an empty status of the scenario's project, created while it is open.
async fn milestone_and_status<B: MessageBody>(app: &impl TestApp<B>, s: &Scenario) -> (u64, u64) {
    let milestone = json(
        app,
        post(
            &format!("{}milestones/", s.project()),
            s.manager,
            json!({ "title": "Appel d'offres", "due_date": "2027-01-15" }),
        ),
    )
    .await["id"]
        .as_u64()
        .unwrap();
    let created = json(
        app,
        post(
            &format!("{}statuses/", s.project()),
            s.manager,
            json!({ "name": "En relecture", "color": "#2563EB", "category": "InProgress", "position": 3 }),
        ),
    )
    .await;
    (milestone, created["status_id"].as_u64().unwrap())
}

#[actix_web::test]
async fn mair_1200_every_write_on_a_closed_project_answers_409() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let s = scenario(&ctx, &app).await;
    let (milestone, status_id) = milestone_and_status(&app, &s).await;
    let before = json(&app, get(&s.project(), s.manager)).await;
    close(&app, &s).await;

    // The manager, and the assignee on their own task, have the rights but the project is closed.
    for (label, request) in writes(&s, s.manager, milestone, status_id) {
        assert_eq!(
            answer(&app, request).await,
            (409, "Project is closed.".to_string()),
            "manager: {label}"
        );
    }
    let assignee_move = patch(&s.task(), s.assignee, json!({ "status": "InProgress" }));
    assert_eq!(status(&app, assignee_move).await, 409);
    let member_comment = post(
        &format!("{}comments/", s.project()),
        s.member,
        json!({ "message": "Vu." }),
    );
    assert_eq!(status(&app, member_comment).await, 409);

    // Nothing changed: same name, same task, same members, still closed.
    let read = json(&app, get(&s.project(), s.manager)).await;
    assert_eq!(read["users_total"], before["users_total"], "{read}");
    assert_eq!(read["project"]["status"], "Completed", "{read}");
    assert_eq!(read["project"]["name"], "Réfection de la place du marché");
    let task = json(&app, get(&s.task(), s.manager)).await;
    assert_eq!(task["name"], "Consulter les riverains", "{task}");
}

#[actix_web::test]
async fn mair_1200_callers_without_rights_keep_their_403_and_404() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let s = scenario(&ctx, &app).await;
    let (milestone, status_id) = milestone_and_status(&app, &s).await;
    close(&app, &s).await;

    // A plain member may not manage the project: 403 before the closed state is considered
    // (comments aside, which any viewer may post, hence the 409 checked above).
    let p = s.project();
    for request in [
        patch(&p, s.member, json!({ "status": "Active" })),
        post(
            &format!("{p}tasks/"),
            s.member,
            json!({ "name": "Sans droit", "fields": [] }),
        ),
        post(
            &format!("{p}users/"),
            s.member,
            json!({ "user_id": s.outsider }),
        ),
        delete(&format!("{p}milestones/{milestone}/"), s.member),
        delete(&format!("{p}statuses/{status_id}/"), s.member),
        delete(&p, s.member),
    ] {
        assert_eq!(status(&app, request).await, 403);
    }
    // Someone who does not see the project still reads 404.
    assert_eq!(
        status(&app, patch(&p, s.outsider, json!({ "status": "Active" }))).await,
        404
    );
    assert_eq!(status(&app, get(&p, s.outsider)).await, 404);
}

#[actix_web::test]
async fn mair_1200_a_closed_project_stays_readable_deletable_and_closable() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let s = scenario(&ctx, &app).await;
    let comment = post(
        &format!("{}comments/", s.project()),
        s.member,
        json!({ "message": "Avant la clôture." }),
    );
    assert_eq!(status(&app, comment).await, 201);
    close(&app, &s).await;

    // Reads: the project, its tasks, members, milestones, statuses and activity.
    let p = s.project();
    for (uri, user) in [
        (p.clone(), s.member),
        (s.task(), s.assignee),
        (format!("{p}tasks/"), s.manager),
        (format!("{p}archived-tasks/"), s.manager),
        (format!("{p}users/"), s.member),
        (format!("{p}milestones/"), s.member),
        (format!("{p}statuses/"), s.member),
        (format!("{p}activity/"), s.member),
    ] {
        assert_eq!(status(&app, get(&uri, user)).await, 200, "{uri}");
    }
    let activity = json(&app, get(&format!("{p}activity/"), s.member)).await;
    assert_eq!(
        activity["items"][0]["message"], "Avant la clôture.",
        "{activity}"
    );

    // Closing again stays idempotent, and the managers may still delete it.
    close(&app, &s).await;
    assert_eq!(status(&app, delete(&p, s.manager)).await, 204);
    assert_eq!(status(&app, get(&p, s.manager)).await, 404);
}

#[actix_web::test]
async fn mair_1200_reopening_gives_every_write_back() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let s = scenario(&ctx, &app).await;
    close(&app, &s).await;
    let p = s.project();
    let new_task = || {
        post(
            &format!("{p}tasks/"),
            s.manager,
            json!({ "name": "Après réouverture", "fields": [] }),
        )
    };
    assert_eq!(status(&app, new_task()).await, 409);

    // A status alone, other than Completed, reopens it; the managers pick the new status.
    assert_eq!(
        status(&app, patch(&p, s.manager, json!({ "status": "Review" }))).await,
        204
    );
    let read = json(&app, get(&p, s.manager)).await;
    assert_eq!(read["project"]["status"], "Review", "{read}");
    assert_eq!(status(&app, new_task()).await, 200);
    assert_eq!(
        status(&app, patch(&p, s.manager, json!({ "name": "Rouvert" }))).await,
        204
    );
    assert_eq!(
        status(
            &app,
            patch(&s.task(), s.assignee, json!({ "status": "InProgress" }))
        )
        .await,
        204
    );

    // A suspended project is not closed: its writes go through.
    assert_eq!(
        status(&app, patch(&p, s.manager, json!({ "status": "Suspended" }))).await,
        204
    );
    assert_eq!(
        status(&app, patch(&p, s.manager, json!({ "name": "En pause" }))).await,
        204
    );
}
