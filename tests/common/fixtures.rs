//! Test set-up helpers. They run as the superuser (`fixtures_db`, MAIR-556): users, roles and
//! groups are outside the grants of `project_api`, which only the code under test connects as.

use std::sync::atomic::{AtomicU64, Ordering};

use super::fixtures_db;
use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// Test set-up SQL (users, roles, groups), without parameters.
#[derive(serde::Serialize, serde::Deserialize)]
struct FixtureSql {
    #[serde(skip)]
    sql: &'static str,
    params: Vec<QueryParam>,
}

impl ApiRequestDto for FixtureSql {
    fn query_sql(&self) -> &'static str {
        self.sql
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Raw SQL without parameters, for test set-up only.
pub fn fixture(sql: String) -> impl ApiRequestDto {
    fixture_sql(sql)
}

fn fixture_sql(sql: String) -> FixtureSql {
    FixtureSql {
        sql: Box::leak(sql.into_boxed_str()),
        params: Vec::new(),
    }
}

/// Unique suffix per call, to isolate the data of a test in the shared database.
pub fn unique_suffix() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{nanos}{}", COUNTER.fetch_add(1, Ordering::Relaxed))
}

/// Syntactically valid argon2id PHC string: `users.password` rejects anything else since database 1.3.0
/// (`chk_users_password_hashed`). Fixture users never log in, so the hash does not need to verify.
const FIXTURE_PASSWORD_HASH: &str = "$argon2id$v=19$m=16,t=2,p=1$dGVzdHNhbHQ$dGVzdGhhc2g";

/// Creates a user (with a role if `role` is given) and returns its id.
pub async fn create_user(first_name: &str, role: Option<&str>) -> u64 {
    let db = fixtures_db().await;
    let suffix = unique_suffix();
    let id: i32 = db
        .fetch_scalar(&fixture_sql(format!(
            "INSERT INTO users (first_name, last_name, email, password) \
             VALUES ('{first_name}', 'Test', '{}.{suffix}@project-api.test', '{FIXTURE_PASSWORD_HASH}') RETURNING id",
            first_name.to_lowercase()
        )))
        .await
        .unwrap();
    if let Some(role) = role {
        db.execute(fixture_sql(format!(
            "INSERT INTO user_roles (user_id, role_id) SELECT {id}, id FROM roles WHERE name = '{role}'"
        )))
        .await
        .unwrap();
    }
    id as u64
}

/// Archives a user: its tokens stay well signed but no longer designate an active account.
pub async fn archive_user(user_id: u64) {
    let db = fixtures_db().await;
    db.execute(fixture_sql(format!(
        "UPDATE users SET is_archived = true WHERE id = {user_id}"
    )))
    .await
    .unwrap();
}

/// Creates a group holding `members` and returns its id.
pub async fn create_group(owner_id: u64, members: &[u64]) -> u64 {
    let db = fixtures_db().await;
    let id: i32 = db
        .fetch_scalar(&fixture_sql(format!(
            "INSERT INTO groups (owner_id, name) VALUES ({owner_id}, 'Groupe {}') RETURNING id",
            unique_suffix()
        )))
        .await
        .unwrap();
    for member in members {
        db.execute(fixture_sql(format!(
            "INSERT INTO group_members (group_id, user_id) VALUES ({id}, {member}) ON CONFLICT DO NOTHING"
        )))
        .await
        .unwrap();
    }
    id as u64
}

/// Forces the status of a task without going through the API (fires the history trigger).
pub async fn set_task_status(task_id: u64, status: &str) {
    let db = fixtures_db().await;
    db.execute(fixture_sql(format!(
        "UPDATE tasks SET status = '{status}'::task_status WHERE id = {task_id}"
    )))
    .await
    .unwrap();
}
