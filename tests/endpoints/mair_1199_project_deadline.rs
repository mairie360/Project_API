//! MAIR-1199: the deadline entered for a project (`projects.due_date`) is set by `deadline` on
//! `POST /projects/` and `PATCH /projects/{id}/`, read back as `deadline` by the detail and the
//! list whatever its tasks, and the date filters of the list use it, else the earliest task due
//! date.

use actix_web::body::MessageBody;
use actix_web::test::TestRequest;
use serde_json::{json, Value};

use super::{get, json, patch, post, status, TestApp, TestContext};
use crate::init_app;

/// `deadline` and `due_date` of the project in the list, and `deadline` in its detail.
async fn dates<B: MessageBody>(
    app: &impl TestApp<B>,
    project_id: u64,
    user: u64,
) -> (Value, Value, Value) {
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
        detail["project"]["deadline"].clone(),
        listed["deadline"].clone(),
        listed["due_date"].clone(),
    )
}

/// Ids of the listed projects matching `query`.
async fn listed<B: MessageBody>(app: &impl TestApp<B>, user: u64, query: &str) -> Vec<u64> {
    let list = json(
        app,
        get(&format!("/api/v1/projects/?limit=500&{query}"), user),
    )
    .await;
    list["projects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|project| project["id"].as_u64().unwrap())
        .collect()
}

#[actix_web::test]
async fn mair_1199_the_deadline_entered_stays_whatever_the_tasks() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let owner = ctx.user("Aaron", Some("Responsable")).await;

    let created = json(
        &app,
        post(
            "/api/v1/projects/",
            owner,
            json!({ "name": "Cantine", "deadline": "2027-06-30" }),
        ),
    )
    .await;
    let project_id = created["project_id"].as_u64().unwrap();
    // Without task: the deadline is read back as entered, `due_date` stays empty.
    assert_eq!(
        dates(&app, project_id, owner).await,
        (json!("2027-06-30"), json!("2027-06-30"), Value::Null)
    );

    // A task due earlier sets `due_date`, never the deadline.
    let task = post(
        &format!("/api/v1/projects/{project_id}/tasks/"),
        owner,
        json!({ "name": "Devis", "fields": [], "due_date": "2027-01-15", "assigned_to": owner }),
    );
    assert_eq!(status(&app, task).await, 200);
    assert_eq!(
        dates(&app, project_id, owner).await,
        (
            json!("2027-06-30"),
            json!("2027-06-30"),
            json!("2027-01-15T00:00:00Z")
        )
    );

    // Absent keeps it, a day replaces it, `null` clears it.
    let p = format!("/api/v1/projects/{project_id}/");
    let renamed = patch(&p, owner, json!({ "name": "Cantine scolaire" }));
    assert_eq!(status(&app, renamed).await, 204);
    assert_eq!(dates(&app, project_id, owner).await.0, json!("2027-06-30"));
    let moved = patch(&p, owner, json!({ "deadline": "2027-09-01" }));
    assert_eq!(status(&app, moved).await, 204);
    assert_eq!(dates(&app, project_id, owner).await.1, json!("2027-09-01"));
    let cleared = patch(&p, owner, json!({ "deadline": null }));
    assert_eq!(status(&app, cleared).await, 204);
    assert_eq!(
        dates(&app, project_id, owner).await,
        (Value::Null, Value::Null, json!("2027-01-15T00:00:00Z"))
    );
}

#[actix_web::test]
async fn mair_1199_the_date_filters_use_the_deadline_else_the_tasks() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let owner = ctx.user("Aaron", Some("Responsable")).await;
    let create = |name: &str, deadline: Value| {
        post(
            "/api/v1/projects/",
            owner,
            json!({ "name": name, "deadline": deadline }),
        )
    };
    let id = |created: Value| created["project_id"].as_u64().unwrap();

    // Deadline in June, but a task due in January: the deadline decides.
    let june = id(json(&app, create("Juin", json!("2027-06-30"))).await);
    let task = post(
        &format!("/api/v1/projects/{june}/tasks/"),
        owner,
        json!({ "name": "Devis", "fields": [], "due_date": "2027-01-15", "assigned_to": owner }),
    );
    assert_eq!(status(&app, task).await, 200);
    // No deadline: the earliest task due date (March) decides.
    let march = id(json(&app, create("Mars", Value::Null)).await);
    let task = post(
        &format!("/api/v1/projects/{march}/tasks/"),
        owner,
        json!({ "name": "Plan", "fields": [], "due_date": "2027-03-10", "assigned_to": owner }),
    );
    assert_eq!(status(&app, task).await, 200);
    // Neither: never matched by a date filter.
    let undated = id(json(&app, create("Sans date", Value::Null)).await);

    let before_april = listed(&app, owner, "due_before=2027-04-01T00:00:00Z").await;
    assert!(before_april.contains(&march), "{before_april:?}");
    assert!(!before_april.contains(&june), "{before_april:?}");
    assert!(!before_april.contains(&undated), "{before_april:?}");

    let after_april = listed(&app, owner, "due_after=2027-04-01T00:00:00Z").await;
    assert!(after_april.contains(&june), "{after_april:?}");
    assert!(!after_april.contains(&march), "{after_april:?}");
    assert!(!after_april.contains(&undated), "{after_april:?}");

    // The deadline day itself is inside a range ending on it.
    let on_the_day = listed(
        &app,
        owner,
        "due_after=2027-06-30T00:00:00Z&due_before=2027-06-30T00:00:00Z",
    )
    .await;
    assert_eq!(on_the_day, vec![june]);
}

#[actix_web::test]
async fn mair_1199_a_deadline_that_is_not_a_day_is_refused_and_changes_nothing() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let owner = ctx.user("Aaron", Some("Responsable")).await;
    let member = ctx.user("Bruno", None).await;

    for deadline in [json!("30/06/2027"), json!("2027-02-30"), json!(20270630)] {
        let refused = post(
            "/api/v1/projects/",
            owner,
            json!({ "name": "Refusé", "deadline": deadline }),
        );
        assert_eq!(status(&app, refused).await, 400, "{deadline}");
    }
    assert!(listed(&app, owner, "search=Refus").await.is_empty());

    let created = json(
        &app,
        post(
            "/api/v1/projects/",
            owner,
            json!({ "name": "Parc", "deadline": "2027-06-30" }),
        ),
    )
    .await;
    let project_id = created["project_id"].as_u64().unwrap();
    let p = format!("/api/v1/projects/{project_id}/");
    let refused = patch(&p, owner, json!({ "deadline": "demain", "name": "Changé" }));
    assert_eq!(status(&app, refused).await, 400);

    // A member who does not manage the project cannot move it.
    let add = post(&format!("{p}users/"), owner, json!({ "user_id": member }));
    assert_eq!(status(&app, add).await, 200);
    let forbidden = patch(&p, member, json!({ "deadline": "2030-01-01" }));
    assert_eq!(status(&app, forbidden).await, 403);

    let project = json(&app, get(&p, member)).await;
    assert_eq!(project["project"]["name"], "Parc");
    assert_eq!(project["project"]["deadline"], "2027-06-30");
}

/// Status and body of a request.
async fn answer<B: MessageBody>(app: &impl TestApp<B>, request: TestRequest) -> (u16, String) {
    let response = actix_web::test::call_service(app, request.to_request()).await;
    let code = response.status().as_u16();
    let body = actix_web::test::read_body(response).await;
    (code, String::from_utf8_lossy(&body).into_owned())
}

/// The promote-staging newman gate expects the `400` of a malformed deadline to name the field,
/// on creation and on update (it answered `Json deserialize error: …` before).
#[actix_web::test]
async fn mair_1203_a_malformed_deadline_names_the_field() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let owner = ctx.user("Aaron", Some("Responsable")).await;

    let (code, text) = answer(
        &app,
        post(
            "/api/v1/projects/",
            owner,
            json!({ "name": "Refusé", "deadline": "30/06/2027" }),
        ),
    )
    .await;
    assert_eq!(code, 400);
    assert!(text.starts_with("Invalid `deadline`: "), "{text}");

    let created = json(
        &app,
        post(
            "/api/v1/projects/",
            owner,
            json!({ "name": "Parc", "deadline": "2027-06-30" }),
        ),
    )
    .await;
    let project_id = created["project_id"].as_u64().unwrap();
    let (code, text) = answer(
        &app,
        patch(
            &format!("/api/v1/projects/{project_id}/"),
            owner,
            json!({ "deadline": "demain" }),
        ),
    )
    .await;
    assert_eq!(code, 400);
    assert!(text.starts_with("Invalid `deadline`: "), "{text}");
}
