use crate::common::get_pool; // Utilisation de ta fonction utilitaire existante
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use project_api::database::project::create::query::create_project_query;
use project_api::database::project::create::view::CreateProjectQueryView;
use project_api::database::users::add_user_to_project::query::add_user_to_project_query;
use project_api::database::users::add_user_to_project::view::AddUserToProjectQueryView;
use project_api::database::users::get_project_users::query::get_project_users_query;
use project_api::database::users::get_project_users::view::GetProjectUsersQueryView;

#[sqlx::test]
async fn test_get_user_from_project_success() {
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

    let result = add_user_to_project_query(view, pool.clone()).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );

    let view = GetProjectUsersQueryView::new(project_id as u64);
    let result = get_project_users_query(view, pool).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
    let users = result.unwrap();
    assert!(
        !users.is_empty(),
        "Expected users to be non-empty, got: {:?}",
        users
    );
}

#[sqlx::test]
async fn test_get_user_from_no_user_project_success() {
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

    let view = GetProjectUsersQueryView::new(project_id as u64);
    let result = get_project_users_query(view, pool).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
    let users = result.unwrap();
    assert!(
        users.is_empty(),
        "Expected users to be empty, got: {:?}",
        users
    );
}

#[sqlx::test]
async fn test_get_users_from_project_unknown_project() {
    let (_container, host) = get_shared_db().await;
    let pool = get_pool(host.to_string()).await;

    let view = GetProjectUsersQueryView::new(999);
    let result = get_project_users_query(view, pool).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
    let users = result.unwrap();
    assert!(
        users.is_empty(),
        "Expected users to be empty, got: {:?}",
        users
    );
}
