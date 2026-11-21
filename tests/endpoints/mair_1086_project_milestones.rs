//! MAIR-1086: the milestones of a project. Whoever sees the project lists them (also embedded in
//! `GET /projects/{id}/`), its managers add, update and delete them; a milestone of another
//! project, or a deleted one, answers `404`.

use serde_json::json;

use super::access::scenario;
use super::{delete, get, json, patch, post, status, TestContext};
use crate::init_app;

#[actix_web::test]
async fn mair_1086_managers_plan_the_milestones_viewers_read_them() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let s = scenario(&ctx, &app).await;
    let milestones = format!("{}milestones/", s.project());

    let late = json(
        &app,
        post(
            &milestones,
            s.manager,
            json!({ "title": " Livraison des travaux ", "due_date": "2027-06-30" }),
        ),
    )
    .await;
    assert_eq!(late["title"], "Livraison des travaux");
    assert_eq!(late["due_date"], "2027-06-30");
    assert_eq!(late["reached"], false);
    assert_eq!(late["project_id"], s.project_id);
    assert_eq!(late["created_by"], s.manager);
    let early = json(
        &app,
        post(
            &milestones,
            s.manager,
            json!({ "title": "Appel d'offres attribué", "due_date": "2026-12-15", "reached": true }),
        ),
    )
    .await;
    assert_eq!(early["reached"], true);
    let late_id = late["id"].as_u64().unwrap();
    let early_id = early["id"].as_u64().unwrap();

    // Every viewer reads them, by due date, in the list and in the project.
    for viewer in [s.manager, s.member, s.assignee] {
        let page = json(&app, get(&milestones, viewer)).await;
        assert_eq!(page["total"], 2);
        assert_eq!(page["milestones"][0]["id"], early_id);
        assert_eq!(page["milestones"][1]["id"], late_id);
        let project = json(&app, get(&s.project(), viewer)).await;
        assert_eq!(project["milestones_total"], 2);
        assert_eq!(project["milestones"][0]["id"], early_id);
    }
    let paged = json(
        &app,
        get(&format!("{milestones}?limit=1&offset=1"), s.member),
    )
    .await;
    assert_eq!(paged["total"], 2);
    assert_eq!(paged["milestones"].as_array().unwrap().len(), 1);
    assert_eq!(paged["milestones"][0]["id"], late_id);

    // A partial update.
    let one = format!("{milestones}{late_id}/");
    let updated = json(&app, patch(&one, s.manager, json!({ "reached": true }))).await;
    assert_eq!(updated["reached"], true);
    assert_eq!(updated["title"], "Livraison des travaux");
    assert_eq!(updated["due_date"], "2027-06-30");
    let moved = json(
        &app,
        patch(
            &one,
            s.manager,
            json!({ "due_date": "2026-11-30", "title": "Livraison" }),
        ),
    )
    .await;
    assert_eq!(moved["due_date"], "2026-11-30");
    assert_eq!(moved["reached"], true);
    let page = json(&app, get(&milestones, s.member)).await;
    assert_eq!(page["milestones"][0]["id"], late_id, "moved first");

    // Invalid bodies.
    for body in [
        json!({ "title": "", "due_date": "2027-01-01" }),
        json!({ "title": "   ", "due_date": "2027-01-01" }),
        json!({ "title": "a".repeat(256), "due_date": "2027-01-01" }),
        json!({ "title": "X" }),
        json!({ "title": "X", "due_date": "01/01/2027" }),
        json!({ "title": "X\u{0}", "due_date": "2027-01-01" }),
    ] {
        assert_eq!(
            status(&app, post(&milestones, s.manager, body.clone())).await,
            400,
            "{body}"
        );
    }
    for body in [json!({ "title": " " }), json!({ "due_date": "2027-02-30" })] {
        assert_eq!(
            status(&app, patch(&one, s.manager, body.clone())).await,
            400,
            "{body}"
        );
    }

    // Only the managers write; an outsider does not see the project.
    let body = json!({ "title": "Refusé", "due_date": "2027-01-01" });
    for refused in [s.member, s.assignee] {
        assert_eq!(
            status(&app, post(&milestones, refused, body.clone())).await,
            403
        );
        assert_eq!(
            status(&app, patch(&one, refused, json!({ "reached": false }))).await,
            403
        );
        assert_eq!(status(&app, delete(&one, refused)).await, 403);
    }
    for outsider in [s.outsider, s.outsider_manager] {
        assert_eq!(status(&app, get(&milestones, outsider)).await, 404);
        assert_eq!(
            status(&app, post(&milestones, outsider, body.clone())).await,
            404
        );
        assert_eq!(
            status(&app, patch(&one, outsider, json!({ "reached": false }))).await,
            404
        );
        assert_eq!(status(&app, delete(&one, outsider)).await, 404);
    }

    // A milestone through another project of the manager is unknown there.
    let other = json(
        &app,
        post(
            "/api/v1/projects/",
            s.manager,
            json!({ "name": "Autre projet" }),
        ),
    )
    .await["project_id"]
        .as_u64()
        .unwrap();
    let crossed = format!("/api/v1/projects/{other}/milestones/{late_id}/");
    assert_eq!(
        status(
            &app,
            patch(&crossed, s.manager, json!({ "reached": false }))
        )
        .await,
        404
    );
    assert_eq!(status(&app, delete(&crossed, s.manager)).await, 404);
    let unknown = format!("{milestones}{}/", i32::MAX);
    assert_eq!(
        status(
            &app,
            patch(&unknown, s.manager, json!({ "reached": false }))
        )
        .await,
        404
    );
    assert_eq!(
        status(&app, get(&format!("{milestones}?limit=-1"), s.manager)).await,
        400
    );
    assert_eq!(
        status(&app, delete(&format!("{milestones}abc/"), s.manager)).await,
        400
    );

    // Deleted: gone from the reads, a second delete or an update is a 404.
    assert_eq!(status(&app, delete(&one, s.manager)).await, 204);
    assert_eq!(status(&app, delete(&one, s.manager)).await, 404);
    assert_eq!(
        status(&app, patch(&one, s.manager, json!({ "reached": false }))).await,
        404
    );
    let page = json(&app, get(&milestones, s.member)).await;
    assert_eq!(page["total"], 1);
    assert_eq!(page["milestones"][0]["id"], early_id);
    let project = json(&app, get(&s.project(), s.member)).await;
    assert_eq!(project["milestones_total"], 1);
}
