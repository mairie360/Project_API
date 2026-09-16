use std::sync::atomic::{AtomicU64, Ordering};

use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};
use mairie360_api_lib::smart_db::SmartDatabase;

/// Requête SQL de mise en place propre aux tests (utilisateurs, rôles, groupes), sans paramètres.
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

fn fixture(sql: String) -> FixtureSql {
    FixtureSql {
        sql: Box::leak(sql.into_boxed_str()),
        params: Vec::new(),
    }
}

/// Suffixe unique par appel, pour isoler les données d'un test dans la base partagée.
pub fn unique_suffix() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{nanos}{}", COUNTER.fetch_add(1, Ordering::Relaxed))
}

/// Crée un utilisateur (avec un rôle si `role` est fourni) et renvoie son identifiant.
pub async fn create_user(db: &SmartDatabase, first_name: &str, role: Option<&str>) -> u64 {
    let suffix = unique_suffix();
    let id: i32 = db
        .fetch_scalar(&fixture(format!(
            "INSERT INTO users (first_name, last_name, email, password) \
             VALUES ('{first_name}', 'Test', '{}.{suffix}@project-api.test', 'password') RETURNING id",
            first_name.to_lowercase()
        )))
        .await
        .unwrap();
    if let Some(role) = role {
        db.execute(fixture(format!(
            "INSERT INTO user_roles (user_id, role_id) SELECT {id}, id FROM roles WHERE name = '{role}'"
        )))
        .await
        .unwrap();
    }
    id as u64
}

/// Crée un groupe contenant `members` et renvoie son identifiant.
pub async fn create_group(db: &SmartDatabase, owner_id: u64, members: &[u64]) -> u64 {
    let id: i32 = db
        .fetch_scalar(&fixture(format!(
            "INSERT INTO groups (owner_id, name) VALUES ({owner_id}, 'Groupe {}') RETURNING id",
            unique_suffix()
        )))
        .await
        .unwrap();
    for member in members {
        db.execute(fixture(format!(
            "INSERT INTO group_members (group_id, user_id) VALUES ({id}, {member}) ON CONFLICT DO NOTHING"
        )))
        .await
        .unwrap();
    }
    id as u64
}

/// Force le statut d'une tâche sans passer par l'API (déclenche le trigger d'historique).
pub async fn set_task_status(db: &SmartDatabase, task_id: u64, status: &str) {
    db.execute(fixture(format!(
        "UPDATE tasks SET status = '{status}'::task_status WHERE id = {task_id}"
    )))
    .await
    .unwrap();
}
