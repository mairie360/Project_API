use crate::common::get_pool; // Utilisation de ta fonction utilitaire existante
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use project_api::database::project::create::query::create_project_query;
use project_api::database::project::create::view::CreateProjectQueryView;
use project_api::database::project::update_status::query::update_project_status_query;
use project_api::database::project::update_status::view::{
    ProjectStatus, UpdateProjectStatusQueryView,
};

#[sqlx::test]
async fn test_update_project_status_to_suspended_success() {
    let (_container, host) = get_shared_db().await;
    let pool = get_pool(host.to_string()).await;

    let view = CreateProjectQueryView::new("Test Project", Some("Test Description"), 1);

    let result = create_project_query(view, pool.clone()).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
    let project_id = result.unwrap() as u64;

    let view = UpdateProjectStatusQueryView::new(project_id, ProjectStatus::Suspended);

    let result = update_project_status_query(view, pool.clone()).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
    let updated_row = result.unwrap() as u64;
    assert!(
        updated_row == 1,
        "Expected updated row count to be 1, got: {}",
        updated_row
    );
}

#[sqlx::test]
async fn test_update_project_status_to_active_success() {
    let (_container, host) = get_shared_db().await;
    let pool = get_pool(host.to_string()).await;

    let view = CreateProjectQueryView::new("Test Project", Some("Test Description"), 1);

    let result = create_project_query(view, pool.clone()).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
    let project_id = result.unwrap() as u64;

    let view = UpdateProjectStatusQueryView::new(project_id, ProjectStatus::Active);

    let result = update_project_status_query(view, pool.clone()).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
    let updated_row = result.unwrap() as u64;
    assert!(
        updated_row == 1,
        "Expected updated row count to be 1, got: {}",
        updated_row
    );
}

#[sqlx::test]
async fn test_update_project_status_to_completed_success() {
    let (_container, host) = get_shared_db().await;
    let pool = get_pool(host.to_string()).await;

    let view = CreateProjectQueryView::new("Test Project", Some("Test Description"), 1);

    let result = create_project_query(view, pool.clone()).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
    let project_id = result.unwrap() as u64;

    let view = UpdateProjectStatusQueryView::new(project_id, ProjectStatus::Completed);

    let result = update_project_status_query(view, pool.clone()).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
    let updated_row = result.unwrap() as u64;
    assert!(
        updated_row == 1,
        "Expected updated row count to be 1, got: {}",
        updated_row
    );
}

#[sqlx::test]
async fn test_update_project_status_unknown_status_error() {
    let (_container, host) = get_shared_db().await;
    let pool = get_pool(host.to_string()).await;

    let view = CreateProjectQueryView::new("Test Project", Some("Test Description"), 1);

    let result = create_project_query(view, pool.clone()).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
    let project_id = result.unwrap() as u64;

    let view = UpdateProjectStatusQueryView::new(project_id, ProjectStatus::Error);

    let result = update_project_status_query(view, pool.clone()).await;

    assert!(
        result.is_err(),
        "Expected result to be Err, got: {:?}",
        result
    );
}
