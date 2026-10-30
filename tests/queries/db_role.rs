//! The code under test connects as `project_api`, never as the superuser (MAIR-556): a superuser
//! ignores the grants of Database's `api_grants.sql`, so a query Project_API may not run in
//! staging would pass here.

use mairie360_api_lib::database::error::{describe_sqlx_error, DbError};
use mairie360_api_lib::error::ApiLibError;
use mairie360_api_lib::smart_db::SmartDatabase;

use crate::common::fixtures::fixture;
use crate::common::{get_smart_db, PROJECT_DB_ROLE};
use crate::endpoints::TestContext;

/// Asserts that `db` is connected as [`PROJECT_DB_ROLE`], without superuser rights.
async fn assert_runs_as_project_api(db: &SmartDatabase) {
    let role: serde_json::Value = db
        .fetch_one(&fixture(
            "SELECT jsonb_build_object('user', current_user, 'superuser', rolsuper) \
             FROM pg_roles WHERE rolname = current_user"
                .to_string(),
        ))
        .await
        .unwrap();
    let user = role["user"].as_str().unwrap();
    let is_superuser = role["superuser"].as_bool().unwrap();

    assert_eq!(user, PROJECT_DB_ROLE);
    assert!(!is_superuser, "{PROJECT_DB_ROLE} must not be a superuser");
}

#[tokio::test]
async fn the_query_views_under_test_run_as_project_api() {
    assert_runs_as_project_api(&get_smart_db().await).await;
}

#[actix_web::test]
async fn the_app_state_under_test_runs_as_project_api() {
    let ctx = TestContext::new().await;

    assert_runs_as_project_api(ctx.db()).await;
}

#[tokio::test]
async fn project_api_is_held_to_its_grants() {
    // Sessions belong to Core: the grants refuse them to project_api, as in staging.
    let error = get_smart_db()
        .await
        .execute(fixture("SELECT 1 FROM sessions LIMIT 1".to_string()))
        .await
        .expect_err("project_api must not read sessions");

    let ApiLibError::Database(DbError::Sqlx(ref inner)) = error else {
        panic!("expected a Postgres error, got {error}");
    };
    // SQLSTATE 42501: insufficient_privilege.
    assert!(
        describe_sqlx_error(inner).starts_with("SQLSTATE 42501"),
        "expected permission denied, got {error}"
    );
}
