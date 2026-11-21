//! MAIR-1085: the budget of a project (`budget_voted`, `budget_committed`, in cents, nullable) is
//! read by the list and the detail, and set by the managers through `PATCH /projects/{id}/`
//! (absent keeps, `null` clears, negative is a `400`).

use serde_json::json;

use super::access::scenario;
use super::{get, json, patch, status, TestContext};
use crate::init_app;

#[actix_web::test]
async fn mair_1085_the_managers_set_the_budget_everyone_reads_it() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let s = scenario(&ctx, &app).await;
    let p = s.project();

    // A new project has no budget.
    let project = json(&app, get(&p, s.manager)).await;
    assert_eq!(project["project"]["budget_voted"], serde_json::Value::Null);
    assert_eq!(
        project["project"]["budget_committed"],
        serde_json::Value::Null
    );

    let set = patch(
        &p,
        s.manager,
        json!({ "budget_voted": 1_250_000, "budget_committed": 480_000 }),
    );
    assert_eq!(status(&app, set).await, 204);
    for viewer in [s.manager, s.member] {
        let project = json(&app, get(&p, viewer)).await;
        assert_eq!(project["project"]["budget_voted"], 1_250_000);
        assert_eq!(project["project"]["budget_committed"], 480_000);
    }
    let list = json(&app, get("/api/v1/projects/?limit=500", s.member)).await;
    let listed = list["projects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|project| project["id"] == s.project_id)
        .unwrap()
        .clone();
    assert_eq!(listed["budget_voted"], 1_250_000, "{list}");
    assert_eq!(listed["budget_committed"], 480_000, "{list}");

    // Absent keeps, `null` clears.
    let renamed = patch(&p, s.manager, json!({ "name": "Place du marché" }));
    assert_eq!(status(&app, renamed).await, 204);
    let cleared = patch(&p, s.manager, json!({ "budget_committed": null }));
    assert_eq!(status(&app, cleared).await, 204);
    let project = json(&app, get(&p, s.manager)).await;
    assert_eq!(project["project"]["budget_voted"], 1_250_000);
    assert_eq!(
        project["project"]["budget_committed"],
        serde_json::Value::Null
    );

    // Negative, fractional or textual amounts are refused and change nothing.
    for body in [
        json!({ "budget_voted": -1 }),
        json!({ "budget_committed": -480_000, "name": "Changé" }),
        json!({ "budget_voted": 12.5 }),
        json!({ "budget_voted": "1000" }),
    ] {
        assert_eq!(
            status(&app, patch(&p, s.manager, body.clone())).await,
            400,
            "{body}"
        );
    }
    let project = json(&app, get(&p, s.manager)).await;
    assert_eq!(project["project"]["budget_voted"], 1_250_000);
    assert_eq!(project["project"]["name"], "Place du marché");

    // Only the managers set it; an outsider does not see the project.
    let refused = patch(&p, s.member, json!({ "budget_voted": 1 }));
    assert_eq!(status(&app, refused).await, 403);
    let refused = patch(&p, s.assignee, json!({ "budget_voted": 1 }));
    assert_eq!(status(&app, refused).await, 403);
    let hidden = patch(&p, s.outsider, json!({ "budget_voted": 1 }));
    assert_eq!(status(&app, hidden).await, 404);
    let project = json(&app, get(&p, s.manager)).await;
    assert_eq!(project["project"]["budget_voted"], 1_250_000);
}
