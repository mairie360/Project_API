//! MAIR-566: task statuses set per project, copied from project templates managed by Admin and
//! Responsable.

use serde_json::{json, Value};
use serial_test::serial;

use super::access::scenario;
use super::{delete, get, json, patch, post, status, TestContext};
use crate::init_app;

/// `name` of each status of a `GET …/statuses/` answer, in order.
fn names(statuses: &Value) -> Vec<String> {
    statuses["statuses"]
        .as_array()
        .unwrap()
        .iter()
        .map(|status| status["name"].as_str().unwrap().to_string())
        .collect()
}

/// Id of the status named `name` in a `GET …/statuses/` answer.
fn status_id(statuses: &Value, name: &str) -> u64 {
    statuses["statuses"]
        .as_array()
        .unwrap()
        .iter()
        .find(|status| status["name"] == name)
        .unwrap_or_else(|| panic!("no status {name}"))["id"]
        .as_u64()
        .unwrap()
}

/// Id of the template named `name`.
fn template_id(templates: &Value, name: &str) -> u64 {
    templates["templates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|template| template["name"] == name)
        .unwrap_or_else(|| panic!("no template {name}"))["id"]
        .as_u64()
        .unwrap()
}

#[actix_web::test]
#[serial]
async fn mair_566_a_project_gets_a_copy_of_the_statuses_of_its_template() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let manager = ctx.user("TemplateUser", Some("Responsable")).await;

    let templates = json(&app, get("/api/v1/project-templates/", manager)).await;
    let first = &templates["templates"][0];
    assert_eq!(first["name"], "Simple");
    assert_eq!(first["is_default"], true);
    for name in ["Avec validation", "Marché public", "Événement"] {
        template_id(&templates, name);
    }

    let public_market = template_id(&templates, "Marché public");
    let created = json(
        &app,
        post(
            "/api/v1/projects/",
            manager,
            json!({ "name": "Marché de la voirie", "template_id": public_market }),
        ),
    )
    .await;
    let statuses_uri = format!(
        "/api/v1/projects/{}/statuses/",
        created["project_id"].as_u64().unwrap()
    );
    let statuses = json(&app, get(&statuses_uri, manager)).await;
    assert_eq!(
        names(&statuses),
        [
            "Étude",
            "Appel d'offres",
            "Attribution",
            "Exécution",
            "Réception"
        ]
    );
    assert_eq!(statuses["statuses"][4]["category"], "Completed");
    assert_eq!(statuses["statuses"][0]["task_count"], 0);

    let plain = json(
        &app,
        post(
            "/api/v1/projects/",
            manager,
            json!({ "name": "Sans modèle" }),
        ),
    )
    .await;
    let plain_statuses = json(
        &app,
        get(
            &format!(
                "/api/v1/projects/{}/statuses/",
                plain["project_id"].as_u64().unwrap()
            ),
            manager,
        ),
    )
    .await;
    assert_eq!(names(&plain_statuses), ["À faire", "En cours", "Terminé"]);

    let unknown = post(
        "/api/v1/projects/",
        manager,
        json!({ "name": "Modèle inconnu", "template_id": 999_999 }),
    );
    assert_eq!(status(&app, unknown).await, 400);
}

#[actix_web::test]
#[serial]
async fn mair_566_a_task_moves_between_the_statuses_of_its_own_project() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let s = scenario(&ctx, &app).await;
    let statuses_uri = format!("/api/v1/projects/{}/statuses/", s.project_id);
    let tasks_uri = format!("/api/v1/projects/{}/tasks/", s.project_id);

    // A status of the project's own (the default template gives À faire, En cours, Terminé).
    let added = json(
        &app,
        post(
            &statuses_uri,
            s.manager,
            json!({ "name": "À valider", "color": "#D97706", "category": "InProgress", "position": 2 }),
        ),
    )
    .await;
    let to_validate = added["status_id"].as_u64().unwrap();
    let statuses = json(&app, get(&statuses_uri, s.member)).await;
    assert_eq!(
        names(&statuses),
        ["À faire", "En cours", "À valider", "Terminé"]
    );

    let created = json(
        &app,
        post(
            &tasks_uri,
            s.manager,
            json!({ "name": "Valider le devis", "fields": [], "status_id": to_validate }),
        ),
    )
    .await;
    let task = format!("{tasks_uri}{}/", created["task_id"].as_u64().unwrap());
    let read = json(&app, get(&task, s.manager)).await;
    assert_eq!(read["status"], "InProgress");
    assert_eq!(read["status_id"], to_validate);
    assert_eq!(read["status_name"], "À valider");

    // The category alone still works: the first status of it.
    let by_category = json(
        &app,
        post(
            &tasks_uri,
            s.manager,
            json!({ "name": "Commander", "fields": [], "status": "InProgress" }),
        ),
    )
    .await;
    let by_category = json(
        &app,
        get(
            &format!("{tasks_uri}{}/", by_category["task_id"].as_u64().unwrap()),
            s.manager,
        ),
    )
    .await;
    assert_eq!(by_category["status_name"], "En cours");

    // A status of another project is refused, nothing is written.
    let other = json(
        &app,
        post(
            "/api/v1/projects/",
            s.manager,
            json!({ "name": "Autre projet" }),
        ),
    )
    .await;
    let other_statuses = json(
        &app,
        get(
            &format!(
                "/api/v1/projects/{}/statuses/",
                other["project_id"].as_u64().unwrap()
            ),
            s.manager,
        ),
    )
    .await;
    let foreign = status_id(&other_statuses, "En cours");
    let refused = post(
        &tasks_uri,
        s.manager,
        json!({ "name": "Mauvais statut", "fields": [], "status_id": foreign }),
    );
    assert_eq!(status(&app, refused).await, 400);
    let refused = patch(&task, s.manager, json!({ "status_id": foreign }));
    assert_eq!(status(&app, refused).await, 400);
    assert_eq!(
        json(&app, get(&task, s.manager)).await["status_id"],
        to_validate
    );

    // The assignee may move their task to another status, and only that.
    let assigned = s.task();
    let done = status_id(&statuses, "Terminé");
    let moved = patch(&assigned, s.assignee, json!({ "status_id": to_validate }));
    assert_eq!(status(&app, moved).await, 204);
    let renamed = patch(
        &assigned,
        s.assignee,
        json!({ "status_id": done, "name": "Renommée" }),
    );
    assert_eq!(status(&app, renamed).await, 403);

    // A completed status archives the task.
    let completed = patch(&task, s.manager, json!({ "status_id": done }));
    assert_eq!(status(&app, completed).await, 204);
    let read = json(&app, get(&task, s.manager)).await;
    assert_eq!(read["status"], "Completed");
    assert!(read["archived_at"].is_string());
    let archived = json(
        &app,
        get(
            &format!("/api/v1/projects/{}/archived-tasks/", s.project_id),
            s.manager,
        ),
    )
    .await;
    assert_eq!(archived["tasks"][0]["status_name"], "Terminé");

    // The cross-project list names the status too.
    let pending = json(&app, get("/api/v1/tasks/?limit=500", s.manager)).await;
    let listed = pending["tasks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|task| task["id"] == s.task_id)
        .expect("the assigned task is pending");
    assert_eq!(listed["status_name"], "À valider");
}

#[actix_web::test]
#[serial]
async fn mair_566_the_project_managers_manage_its_statuses() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let s = scenario(&ctx, &app).await;
    let statuses_uri = format!("/api/v1/projects/{}/statuses/", s.project_id);
    let statuses = json(&app, get(&statuses_uri, s.manager)).await;
    let (todo, in_progress, done) = (
        status_id(&statuses, "À faire"),
        status_id(&statuses, "En cours"),
        status_id(&statuses, "Terminé"),
    );

    // Only the managers write; an outsider does not even see them.
    let body = json!({ "name": "Bloqué", "category": "InProgress" });
    assert_eq!(
        status(&app, post(&statuses_uri, s.member, body.clone())).await,
        403
    );
    assert_eq!(status(&app, get(&statuses_uri, s.outsider)).await, 404);
    for invalid in [
        json!({ "name": " ", "category": "InProgress" }),
        json!({ "name": "Bloqué", "category": "Review" }),
        json!({ "name": "Bloqué", "color": "red", "category": "InProgress" }),
    ] {
        let refused = post(&statuses_uri, s.manager, invalid.clone());
        assert_eq!(status(&app, refused).await, 400, "{invalid}");
    }
    let taken = post(
        &statuses_uri,
        s.manager,
        json!({ "name": "en cours", "category": "InProgress" }),
    );
    assert_eq!(status(&app, taken).await, 409);

    let blocked = json(&app, post(&statuses_uri, s.manager, body)).await["status_id"]
        .as_u64()
        .unwrap();
    let blocked_uri = format!("{statuses_uri}{blocked}/");

    // Rename and move first: the others shift.
    let moved = patch(
        &blocked_uri,
        s.manager,
        json!({ "name": "En attente", "position": 0 }),
    );
    assert_eq!(status(&app, moved).await, 204);
    assert_eq!(
        names(&json(&app, get(&statuses_uri, s.manager)).await),
        ["En attente", "À faire", "En cours", "Terminé"]
    );

    // A category keeps at least one status.
    let last_todo = patch(
        &format!("{statuses_uri}{todo}/"),
        s.manager,
        json!({ "category": "InProgress" }),
    );
    assert_eq!(status(&app, last_todo).await, 409);
    let last_done = delete(&format!("{statuses_uri}{done}/"), s.manager);
    assert_eq!(status(&app, last_done).await, 409);

    // A status holding tasks needs a status to move them to.
    let assigned = patch(&s.task(), s.manager, json!({ "status_id": in_progress }));
    assert_eq!(status(&app, assigned).await, 204);
    let in_progress_uri = format!("{statuses_uri}{in_progress}/");
    assert_eq!(status(&app, delete(&in_progress_uri, s.manager)).await, 409);
    let to_self = delete(
        &format!("{in_progress_uri}?move_to={in_progress}"),
        s.manager,
    );
    assert_eq!(status(&app, to_self).await, 400);
    let moved = delete(&format!("{in_progress_uri}?move_to={blocked}"), s.manager);
    assert_eq!(status(&app, moved).await, 204);
    let task = json(&app, get(&s.task(), s.manager)).await;
    assert_eq!(task["status_name"], "En attente");
    assert_eq!(
        names(&json(&app, get(&statuses_uri, s.manager)).await),
        ["En attente", "À faire", "Terminé"]
    );
    assert_eq!(status(&app, delete(&in_progress_uri, s.manager)).await, 404);

    // Recategorizing a status moves its tasks: Completed archives them.
    let recategorized = patch(&blocked_uri, s.manager, json!({ "category": "Completed" }));
    // `En attente` is the last InProgress status now.
    assert_eq!(status(&app, recategorized).await, 409);
    let review = json(
        &app,
        post(
            &statuses_uri,
            s.manager,
            json!({ "name": "Relecture", "category": "InProgress" }),
        ),
    )
    .await;
    assert!(review["status_id"].is_u64());
    let recategorized = patch(&blocked_uri, s.manager, json!({ "category": "Completed" }));
    assert_eq!(status(&app, recategorized).await, 204);
    let task = json(&app, get(&s.task(), s.manager)).await;
    assert_eq!(task["status"], "Completed");
    assert!(task["archived_at"].is_string());
}

#[actix_web::test]
#[serial]
async fn mair_566_templates_are_managed_by_admin_and_responsable_only() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let responsable = ctx.user("TemplateResponsable", Some("Responsable")).await;
    let admin = ctx.user("TemplateAdmin", Some("Admin")).await;
    let maire = ctx.user("TemplateMaire", Some("Maire")).await;
    let agent = ctx.user("TemplateAgent", None).await;
    let name = format!("Travaux de voirie {responsable}");
    let body = json!({
        "name": name,
        "description": "Étude, chantier, réception.",
        "statuses": [
            { "name": "Étude", "category": "Todo" },
            { "name": "Chantier", "color": "#D97706", "category": "InProgress" },
            { "name": "Réception", "color": "#16A34A", "category": "Completed" }
        ],
        "custom_fields": [{ "label": "Budget", "task_type": "select", "fields_options": [] }]
    });

    for refused in [maire, agent] {
        let created = post("/api/v1/project-templates/", refused, body.clone());
        assert_eq!(status(&app, created).await, 403);
    }
    let mut missing_done = body.clone();
    missing_done["statuses"].as_array_mut().unwrap().pop();
    let created = post("/api/v1/project-templates/", responsable, missing_done);
    assert_eq!(status(&app, created).await, 400);

    let created = json(
        &app,
        post("/api/v1/project-templates/", responsable, body.clone()),
    )
    .await;
    let template = format!(
        "/api/v1/project-templates/{}/",
        created["template_id"].as_u64().unwrap()
    );
    let duplicate = post("/api/v1/project-templates/", admin, body.clone());
    assert_eq!(status(&app, duplicate).await, 409);

    // Anyone signed in reads it.
    let read = json(&app, get(&template, agent)).await;
    assert_eq!(read["statuses"][1]["name"], "Chantier");
    assert_eq!(read["custom_fields"][0]["label"], "Budget");

    // A project created from it gets a copy: editing the template does not change it.
    let project = json(
        &app,
        post(
            "/api/v1/projects/",
            responsable,
            json!({ "name": "Rue de la mairie", "template_id": created["template_id"] }),
        ),
    )
    .await;
    let edited = patch(
        &template,
        admin,
        json!({ "statuses": [
            { "name": "À étudier", "category": "Todo" },
            { "name": "Chantier", "category": "InProgress" },
            { "name": "Réception", "category": "Completed" }
        ] }),
    );
    assert_eq!(status(&app, edited).await, 204);
    assert_eq!(
        json(&app, get(&template, agent)).await["statuses"][0]["name"],
        "À étudier"
    );
    let project_statuses = json(
        &app,
        get(
            &format!(
                "/api/v1/projects/{}/statuses/",
                project["project_id"].as_u64().unwrap()
            ),
            responsable,
        ),
    )
    .await;
    assert_eq!(names(&project_statuses), ["Étude", "Chantier", "Réception"]);
    assert_eq!(
        status(&app, patch(&template, agent, json!({ "name": "X" }))).await,
        403
    );

    // The default template stays; another one can go.
    let templates = json(&app, get("/api/v1/project-templates/", agent)).await;
    let simple = template_id(&templates, "Simple");
    let default = delete(&format!("/api/v1/project-templates/{simple}/"), admin);
    assert_eq!(status(&app, default).await, 409);
    assert_eq!(status(&app, delete(&template, agent)).await, 403);
    assert_eq!(status(&app, delete(&template, responsable)).await, 204);
    assert_eq!(status(&app, get(&template, agent)).await, 404);
    assert_eq!(
        names(
            &json(
                &app,
                get(
                    &format!(
                        "/api/v1/projects/{}/statuses/",
                        project["project_id"].as_u64().unwrap()
                    ),
                    responsable,
                ),
            )
            .await
        ),
        ["Étude", "Chantier", "Réception"]
    );
}
