use crate::common::get_smart_db;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use project_api::database::project::create::view::CreateProjectQueryView;
use project_api::database::users::add_user_to_project::view::AddUserToProjectQueryView;
use project_api::database::users::get_project_users::view::GetProjectUsersQueryView;

#[tokio::test]
async fn test_get_user_from_project_success() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;

    let view = CreateProjectQueryView::new("Test Project", Some("Test Description"), 1);
    let project_id = db.fetch_scalar::<i32, _>(&view).await.unwrap() as u64;

    let view = AddUserToProjectQueryView::new(project_id, 2);
    assert!(db.execute(view).await.is_ok());

    let view = GetProjectUsersQueryView::new(project_id);
    let result = db.fetch_all::<i32, _>(&view).await;

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

#[tokio::test]
async fn test_get_user_from_no_user_project_success() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;

    let view = CreateProjectQueryView::new("Test Project", Some("Test Description"), 1);
    let project_id = db.fetch_scalar::<i32, _>(&view).await.unwrap() as u64;

    let view = GetProjectUsersQueryView::new(project_id);
    let result = db.fetch_all::<i32, _>(&view).await;

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

#[tokio::test]
async fn test_get_users_from_project_unknown_project() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;

    let view = GetProjectUsersQueryView::new(999);
    let result = db.fetch_all::<i32, _>(&view).await;

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
