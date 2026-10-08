//! MAIR-288: `access-matrix.yaml` covers every operation of the `OpenAPI`, and the API answers as
//! it says. Each operation is called without a session, as a user of each role who has no relation
//! to the project, and as its owner, a member and the task's assignee: an allowed caller never gets
//! 401 / 403 / 404, any other caller gets 403, or 404 when the project is not visible to them.

use std::collections::{BTreeMap, BTreeSet};

use actix_web::body::MessageBody;
use actix_web::test::TestRequest;
use project_api::endpoints::swagger::ApiDoc;
use serde::Deserialize;
use serde_json::{json, Value};
use serial_test::serial;
use utoipa::OpenApi;

use super::{as_user, json, post, status, TestApp, TestContext};
use crate::init_app;

const MATRIX: &str = include_str!("../../access-matrix.yaml");
const ROLES: [&str; 5] = ["Admin", "Maire", "Responsable", "User", "Guest"];
const RELATIONS: [&str; 3] = ["owner", "member", "assignee"];
/// The Admin seeded by the database migrations.
const ADMIN_ID: u64 = 1;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Matrix {
    version: u32,
    api: String,
    roles: Vec<String>,
    operations: BTreeMap<String, Operation>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Operation {
    access: String,
    #[serde(default)]
    roles: Vec<String>,
    #[serde(default)]
    ownership: Vec<String>,
    #[serde(default)]
    personal: Vec<String>,
    #[allow(dead_code)]
    note: Option<String>,
}

fn matrix() -> Matrix {
    yaml_serde::from_str(MATRIX).expect("access-matrix.yaml is valid")
}

/// The spec the API serves (`ApiDoc`, the contract the BFF is generated from), as JSON.
fn spec() -> Value {
    serde_json::to_value(ApiDoc::openapi()).unwrap()
}

/// "METHOD /path" of every operation of the spec.
fn spec_operations(spec: &Value) -> BTreeSet<String> {
    let mut operations = BTreeSet::new();
    for (path, item) in spec["paths"].as_object().unwrap() {
        for method in ["get", "post", "put", "patch", "delete"] {
            if item.get(method).is_some() {
                operations.insert(format!("{} {path}", method.to_uppercase()));
            }
        }
    }
    operations
}

#[test]
fn the_matrix_covers_every_operation_of_the_spec() {
    let matrix = matrix();
    assert_eq!((matrix.version, matrix.api.as_str()), (1, "project"));
    assert_eq!(matrix.roles, ROLES);
    let spec = spec_operations(&spec());
    let listed: BTreeSet<String> = matrix.operations.keys().cloned().collect();
    let missing: Vec<_> = spec.difference(&listed).collect();
    let unknown: Vec<_> = listed.difference(&spec).collect();
    assert!(
        missing.is_empty(),
        "operations of the OpenAPI missing from access-matrix.yaml: {missing:?}"
    );
    assert!(
        unknown.is_empty(),
        "operations of access-matrix.yaml that the OpenAPI does not have: {unknown:?}"
    );
    for (name, op) in &matrix.operations {
        assert!(
            ["public", "authenticated"].contains(&op.access.as_str()),
            "{name}: access"
        );
        assert!(
            op.roles.iter().all(|r| ROLES.contains(&r.as_str())),
            "{name}: unknown role"
        );
        assert!(
            op.ownership.iter().all(|o| RELATIONS.contains(&o.as_str())),
            "{name}: ownership"
        );
        if op.access == "authenticated" {
            assert!(
                !op.roles.is_empty() || !op.ownership.is_empty(),
                "{name}: nobody may call it"
            );
        }
        assert!(
            op.personal.iter().all(|f| !f.is_empty()),
            "{name}: personal"
        );
    }
}

/// The people around one project, created once.
struct People {
    owner: u64,
    member: u64,
    assignee: u64,
}

/// Fresh instances for one call: a project owned by `owner` with `member` and `assignee` as
/// members, a task assigned to `assignee`, and an extra member to remove.
struct Fixtures {
    project: u64,
    task: u64,
    extra: u64,
}

async fn fixtures<B: MessageBody>(
    ctx: &TestContext,
    app: &impl TestApp<B>,
    people: &People,
) -> Fixtures {
    let extra = ctx.user("Extra", Some("User")).await;
    let created = json(
        app,
        post(
            "/api/v1/projects/",
            people.owner,
            json!({ "name": "Matrice d'accès" }),
        ),
    )
    .await;
    let project = created["project_id"].as_u64().unwrap();
    for user in [people.member, people.assignee, extra] {
        let got = status(
            app,
            post(
                &format!("/api/v1/projects/{project}/users/"),
                people.owner,
                json!({ "user_id": user }),
            ),
        )
        .await;
        assert_eq!(got, 200, "fixture: add {user} to project {project}");
    }
    let task = json(
        app,
        post(
            &format!("/api/v1/projects/{project}/tasks/"),
            people.owner,
            json!({ "name": "Tâche de la matrice", "fields": [], "assigned_to": people.assignee }),
        ),
    )
    .await;
    Fixtures {
        project,
        task: task["task_id"].as_u64().unwrap(),
        extra,
    }
}

/// The request of one operation on the fixtures: path parameters, and the example body of the
/// spec, made valid for the fixtures (no foreign ids).
fn request(spec: &Value, name: &str, f: &Fixtures) -> TestRequest {
    let (method, path) = name.split_once(' ').unwrap();
    let uri = path
        .replace("{project_id}", &f.project.to_string())
        .replace("{task_id}", &f.task.to_string())
        .replace("{user_id}", &f.extra.to_string());
    let example = spec["paths"][path][method.to_lowercase()]["requestBody"]["content"]
        ["application/json"]["example"]
        .clone();
    let body = match name {
        "POST /api/v1/projects/" => Some(json!({ "name": "Projet de la matrice" })),
        "POST /api/v1/projects/{project_id}/tasks/" => {
            Some(json!({ "name": "Nouvelle tâche", "fields": [] }))
        }
        "POST /api/v1/projects/{project_id}/users/" => Some(json!({ "user_id": f.extra })),
        _ if example.is_null() => None,
        _ => Some(example),
    };
    let request = match method {
        "GET" => TestRequest::get(),
        "POST" => TestRequest::post(),
        "PUT" => TestRequest::put(),
        "PATCH" => TestRequest::patch(),
        "DELETE" => TestRequest::delete(),
        other => panic!("{other}"),
    }
    .uri(&uri);
    match body {
        Some(body) => request.set_json(body),
        None => request,
    }
}

#[actix_web::test]
#[serial]
async fn every_role_and_relation_gets_what_the_matrix_says() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let spec = spec();
    let people = People {
        owner: ctx.user("Owner", Some("Responsable")).await,
        member: ctx.user("Member", Some("User")).await,
        assignee: ctx.user("Assignee", Some("User")).await,
    };
    // Users of each role with no relation to the projects (the schema also gives every account
    // the Guest role).
    let mut callers: Vec<(String, u64)> = vec![("Admin".to_string(), ADMIN_ID)];
    for role in &ROLES[1..] {
        let id = ctx
            .user(
                &format!("Matrix{role}"),
                (*role != "Guest").then_some(*role),
            )
            .await;
        callers.push(((*role).to_string(), id));
    }

    let mut failures = Vec::new();
    for (name, op) in &matrix().operations {
        if op.access == "public" {
            continue;
        }
        let f = fixtures(&ctx, &app, &people).await;
        let anonymous = status(&app, request(&spec, name, &f)).await;
        if anonymous != 401 {
            failures.push(format!(
                "{name}: got {anonymous} without a session, expected 401"
            ));
        }
        let mut cases: Vec<(String, u64, bool)> = callers
            .iter()
            .map(|(role, id)| (role.clone(), *id, op.roles.contains(role)))
            .collect();
        // The owner is a Responsable, the member and the assignee plain agents; the assignee is
        // also a member of the project.
        for (who, user, role, relations) in [
            ("owner", people.owner, "Responsable", &["owner"][..]),
            ("member", people.member, "User", &["member"][..]),
            (
                "assignee",
                people.assignee,
                "User",
                &["assignee", "member"][..],
            ),
        ] {
            let allowed = op.roles.iter().any(|r| r == role)
                || op.ownership.iter().any(|o| relations.contains(&o.as_str()));
            cases.push((who.to_string(), user, allowed));
        }
        for (who, user, allowed) in cases {
            let mut f = fixtures(&ctx, &app, &people).await;
            if name == "POST /api/v1/projects/{project_id}/users/" {
                // The extra member is already in: add another fresh account instead.
                f.extra = ctx.user("Joiner", Some("User")).await;
            }
            let got = status(&app, as_user(request(&spec, name, &f), user)).await;
            let refused = got == 401 || got == 403 || got == 404;
            if allowed && refused {
                failures.push(format!(
                    "{name}: {who} is allowed by the matrix but got {got}"
                ));
            }
            if !allowed && got != 403 && got != 404 {
                failures.push(format!(
                    "{name}: {who} is not allowed by the matrix but got {got}, expected 403 or 404"
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "the API does not answer as access-matrix.yaml says:\n{}",
        failures.join("\n")
    );
}
