//! MAIR-691: an Admin manages every project template, a Responsable only the ones they created
//! and never the default one. Each template tells the caller whether it may manage it
//! (`can_manage`), so that a front hides the actions that would be refused.

use actix_web::body::MessageBody;
use serde_json::{json, Value};

use super::{delete, get, json, patch, post, status, TestApp, TestContext};
use crate::init_app;

const TEMPLATES: &str = "/api/v1/project-templates/";

fn body(name: &str) -> Value {
    json!({
        "name": name,
        "description": "Étude, chantier, réception.",
        "statuses": [
            { "name": "Étude", "category": "Todo" },
            { "name": "Chantier", "category": "InProgress" },
            { "name": "Réception", "category": "Completed" }
        ],
        "custom_fields": []
    })
}

fn uri(template: u64) -> String {
    format!("{TEMPLATES}{template}/")
}

/// `can_manage` of the template `id` in a `GET /project-templates/` answer.
fn can_manage(templates: &Value, id: u64) -> bool {
    templates["templates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|template| template["id"] == id)
        .unwrap_or_else(|| panic!("no template {id}"))["can_manage"]
        .as_bool()
        .unwrap()
}

async fn default_template<B: MessageBody>(app: &impl TestApp<B>, user: u64) -> u64 {
    let templates = json(app, get(TEMPLATES, user)).await;
    let first = &templates["templates"][0];
    assert_eq!(first["is_default"], true);
    first["id"].as_u64().unwrap()
}

#[actix_web::test]
async fn mair_691_a_responsable_cannot_modify_or_delete_the_template_of_another_responsable() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let author = ctx.user("TemplateAuthor", Some("Responsable")).await;
    let other = ctx.user("TemplateOther", Some("Responsable")).await;
    let admin = ctx.user("TemplateAdmin", Some("Admin")).await;
    let created = json(&app, post(TEMPLATES, author, body("Voirie de A"))).await;
    let template = created["template_id"].as_u64().unwrap();

    // Another Responsable: refused, and the template is unchanged.
    let renamed = patch(&uri(template), other, json!({ "name": "Voirie de B" }));
    assert_eq!(status(&app, renamed).await, 403);
    let statuses = patch(
        &uri(template),
        other,
        json!({ "statuses": [
            { "name": "A", "category": "Todo" },
            { "name": "B", "category": "InProgress" },
            { "name": "C", "category": "Completed" }
        ] }),
    );
    assert_eq!(status(&app, statuses).await, 403);
    assert_eq!(status(&app, delete(&uri(template), other)).await, 403);
    let read = json(&app, get(&uri(template), other)).await;
    assert_eq!(read["name"], "Voirie de A");
    assert_eq!(read["statuses"][0]["name"], "Étude");
    assert_eq!(read["can_manage"], false);

    // Its author, then an Admin, may.
    let own = get(&uri(template), author);
    assert_eq!(json(&app, own).await["can_manage"], true);
    let renamed = patch(&uri(template), author, json!({ "name": "Voirie de A (2)" }));
    assert_eq!(status(&app, renamed).await, 204);
    let by_admin = patch(&uri(template), admin, json!({ "name": "Voirie corrigée" }));
    assert_eq!(status(&app, by_admin).await, 204);
    assert_eq!(
        json(&app, get(&uri(template), admin)).await["name"],
        "Voirie corrigée"
    );
    assert_eq!(status(&app, delete(&uri(template), other)).await, 403);
    assert_eq!(status(&app, delete(&uri(template), author)).await, 204);

    // An Admin deletes the template of a Responsable.
    let created = json(&app, post(TEMPLATES, author, body("Espaces verts"))).await;
    let template = created["template_id"].as_u64().unwrap();
    assert_eq!(status(&app, delete(&uri(template), admin)).await, 204);
    assert_eq!(status(&app, get(&uri(template), admin)).await, 404);
}

#[actix_web::test]
async fn mair_691_the_default_template_is_modified_by_an_admin_only() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let responsable = ctx.user("DefaultResponsable", Some("Responsable")).await;
    let admin = ctx.user("DefaultAdmin", Some("Admin")).await;
    let default = default_template(&app, responsable).await;

    let renamed = patch(&uri(default), responsable, json!({ "name": "Mon modèle" }));
    assert_eq!(status(&app, renamed).await, 403);
    let statuses = patch(
        &uri(default),
        responsable,
        json!({ "statuses": [
            { "name": "A", "category": "Todo" },
            { "name": "B", "category": "InProgress" },
            { "name": "C", "category": "Completed" }
        ] }),
    );
    assert_eq!(status(&app, statuses).await, 403);
    // Refused for the right reason: not the default's 409 of an Admin.
    assert_eq!(status(&app, delete(&uri(default), responsable)).await, 403);
    let unchanged = json(&app, get(&uri(default), responsable)).await;
    assert_eq!(unchanged["name"], "Simple");
    assert_eq!(unchanged["can_manage"], false);

    let by_admin = patch(
        &uri(default),
        admin,
        json!({ "description": "Modèle par défaut" }),
    );
    assert_eq!(status(&app, by_admin).await, 204);
    assert_eq!(
        json(&app, get(&uri(default), admin)).await["description"],
        "Modèle par défaut"
    );
    // It stays undeletable, whoever asks.
    assert_eq!(status(&app, delete(&uri(default), admin)).await, 409);
}

#[actix_web::test]
async fn mair_691_can_manage_tells_each_caller_what_it_may_do_with_each_template() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let author = ctx.user("ManageAuthor", Some("Responsable")).await;
    let other = ctx.user("ManageOther", Some("Responsable")).await;
    let admin = ctx.user("ManageAdmin", Some("Admin")).await;
    let agent = ctx.user("ManageAgent", None).await;
    let created = json(&app, post(TEMPLATES, author, body("Modèle de A"))).await;
    let template = created["template_id"].as_u64().unwrap();
    let default = default_template(&app, author).await;

    for (caller, own, default_one, label) in [
        (author, true, false, "author"),
        (other, false, false, "other Responsable"),
        (admin, true, true, "Admin"),
        (agent, false, false, "agent"),
    ] {
        let templates = json(&app, get(TEMPLATES, caller)).await;
        assert_eq!(can_manage(&templates, template), own, "{label}: own");
        assert_eq!(
            can_manage(&templates, default),
            default_one,
            "{label}: default"
        );
        let one = json(&app, get(&uri(template), caller)).await;
        assert_eq!(one["can_manage"], own, "{label}: one template");
    }
    // The seeded templates belong to the system admin: no Responsable manages them.
    let templates = json(&app, get(TEMPLATES, author)).await;
    for seeded in templates["templates"].as_array().unwrap() {
        if seeded["id"] != template {
            assert_eq!(seeded["can_manage"], false, "{}", seeded["name"]);
        }
    }
}

#[actix_web::test]
async fn mair_691_a_template_without_author_is_managed_by_an_admin_only() {
    use crate::common::fixtures::fixture;
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let author = ctx.user("OrphanAuthor", Some("Responsable")).await;
    let admin = ctx.user("OrphanAdmin", Some("Admin")).await;
    let created = json(&app, post(TEMPLATES, author, body("Modèle orphelin"))).await;
    let template = created["template_id"].as_u64().unwrap();
    crate::common::fixtures_db()
        .await
        .execute(fixture(format!(
            "UPDATE project_templates SET created_by = NULL WHERE id = {template}"
        )))
        .await
        .unwrap();

    let read = json(&app, get(&uri(template), author)).await;
    assert_eq!(read["can_manage"], false);
    let renamed = patch(&uri(template), author, json!({ "name": "Repris" }));
    assert_eq!(status(&app, renamed).await, 403);
    assert_eq!(status(&app, delete(&uri(template), author)).await, 403);
    let by_admin = patch(
        &uri(template),
        admin,
        json!({ "name": "Repris par l'Admin" }),
    );
    assert_eq!(status(&app, by_admin).await, 204);
    assert_eq!(status(&app, delete(&uri(template), admin)).await, 204);
}

#[actix_web::test]
async fn mair_691_an_unknown_template_stays_a_404_for_a_manager_and_a_403_for_others() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let responsable = ctx.user("UnknownResponsable", Some("Responsable")).await;
    let agent = ctx.user("UnknownAgent", None).await;
    let patch_unknown = patch(&uri(999_999), responsable, json!({ "name": "X" }));
    assert_eq!(status(&app, patch_unknown).await, 404);
    assert_eq!(status(&app, delete(&uri(999_999), responsable)).await, 404);
    assert_eq!(status(&app, delete(&uri(999_999), agent)).await, 403);
}
