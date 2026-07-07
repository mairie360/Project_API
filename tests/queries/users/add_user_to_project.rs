use crate::common::get_pool; // Utilisation de ta fonction utilitaire existante
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use project_api::database::project::create::query::create_project_query;
use project_api::database::project::create::view::CreateProjectQueryView;
use project_api::database::users::add_user_to_project::query::add_user_to_project_query;
use project_api::database::users::add_user_to_project::view::AddUserToProjectQueryView;

#[sqlx::test]
async fn test_add_user_to_project_success() {
    let (_container, host) = get_shared_db().await;
    let pool = get_pool(host.to_string()).await;

    let view = CreateProjectQueryView::new("Test Project", Some("Test Description"), 1);

    let result = create_project_query(view, pool.clone()).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
    let project_id = result.unwrap();
    assert!(
        project_id != 0,
        "Expected project_id to be non-zero, got: {}",
        project_id
    );

    let view = AddUserToProjectQueryView::new(project_id as u64, 2);

    let result = add_user_to_project_query(view, pool).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
}
#[sqlx::test]
async fn test_add_user_to_project_unknown_user() {
    let (_container, host) = get_shared_db().await;
    let pool = get_pool(host.to_string()).await;

    let view = CreateProjectQueryView::new("Test Project", Some("Test Description"), 1);

    let result = create_project_query(view, pool.clone()).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
    let project_id = result.unwrap();
    assert!(
        project_id != 0,
        "Expected project_id to be non-zero, got: {}",
        project_id
    );

    let view = AddUserToProjectQueryView::new(project_id as u64, 999);

    let result = add_user_to_project_query(view, pool).await;

    assert!(
        result.is_err(),
        "Expected result to be Err, got: {:?}",
        result
    );
}

#[sqlx::test]
async fn test_add_user_to_project_unknown_project() {
    let (_container, host) = get_shared_db().await;
    let pool = get_pool(host.to_string()).await;

    let view = AddUserToProjectQueryView::new(999, 2);

    let result = add_user_to_project_query(view, pool).await;

    assert!(
        result.is_err(),
        "Expected result to be Err, got: {:?}",
        result
    );
}

#[sqlx::test]
async fn test_add_user_to_project_unknown_project_and_unknown_user() {
    let (_container, host) = get_shared_db().await;
    let pool = get_pool(host.to_string()).await;

    let view = AddUserToProjectQueryView::new(999, 999);

    let result = add_user_to_project_query(view, pool).await;

    assert!(
        result.is_err(),
        "Expected result to be Err, got: {:?}",
        result
    );
}
