//! MAIR-1084: a project can be put on hold. The API calls it `Suspended` (`project_status`
//! `suspended`): accepted at creation and by `PATCH`, kept by the list filter and counted by the
//! summary of `GET /projects/`, and taken back to `Active` when the work resumes.

use serde_json::json;

use super::access::scenario;
use super::{get, json, patch, post, status, TestContext};
use crate::init_app;

fn listed_ids(page: &serde_json::Value) -> Vec<u64> {
    page["projects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|project| project["id"].as_u64().unwrap())
        .collect()
}

#[actix_web::test]
async fn mair_1084_a_project_is_put_on_hold_and_resumed() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let s = scenario(&ctx, &app).await;

    // Created on hold.
    let created = post(
        "/api/v1/projects/",
        s.manager,
        json!({ "name": "Extension de la médiathèque", "status": "Suspended" }),
    );
    let on_hold = json(&app, created).await["project_id"].as_u64().unwrap();
    let p = format!("/api/v1/projects/{on_hold}/");
    assert_eq!(
        json(&app, get(&p, s.manager)).await["project"]["status"],
        "Suspended"
    );

    // The filter keeps it, the summary counts it, an Active filter leaves it out.
    let suspended = json(
        &app,
        get("/api/v1/projects/?status=Suspended&limit=500", s.manager),
    )
    .await;
    assert!(listed_ids(&suspended).contains(&on_hold), "{suspended}");
    assert!(
        !listed_ids(&suspended).contains(&s.project_id),
        "{suspended}"
    );
    assert_eq!(suspended["summary"]["by_status"]["suspended"], 1);
    assert_eq!(suspended["total"], 1);
    let every = json(&app, get("/api/v1/projects/?limit=500", s.manager)).await;
    assert_eq!(every["summary"]["by_status"]["suspended"], 1, "{every}");
    let active = json(
        &app,
        get("/api/v1/projects/?status=Active&limit=500", s.manager),
    )
    .await;
    assert!(!listed_ids(&active).contains(&on_hold), "{active}");

    // Resumed, then put on hold again by a PATCH.
    assert_eq!(
        status(&app, patch(&p, s.manager, json!({ "status": "Active" }))).await,
        204
    );
    assert_eq!(
        json(&app, get(&p, s.manager)).await["project"]["status"],
        "Active"
    );
    let running = format!("/api/v1/projects/{}/", s.project_id);
    assert_eq!(
        status(
            &app,
            patch(&running, s.manager, json!({ "status": "Suspended" }))
        )
        .await,
        204
    );
    let suspended = json(
        &app,
        get("/api/v1/projects/?status=Suspended&limit=500", s.manager),
    )
    .await;
    assert_eq!(listed_ids(&suspended), vec![s.project_id], "{suspended}");
    assert_eq!(suspended["projects"][0]["status"], "Suspended");

    // A plain member sees the status but cannot change it.
    assert_eq!(
        json(&app, get(&running, s.member)).await["project"]["status"],
        "Suspended"
    );
    assert_eq!(
        status(
            &app,
            patch(&running, s.member, json!({ "status": "Active" }))
        )
        .await,
        403
    );
}
