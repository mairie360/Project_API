use crate::common::get_smart_db;
use project_api::database::project::create::view::CreateProjectQueryView;
use project_api::database::users::add_user_to_project::view::AddUserToProjectQueryView;
use project_api::database::users::remove_user_from_project::view::RemoveUserFromProjectQueryView;

#[tokio::test]
async fn test_remove_user_from_project_success() {
    let db = get_smart_db().await;

    let view = CreateProjectQueryView::new("Test Project", Some("Test Description"), 1);
    let project_id = db.fetch_scalar::<i32, _>(&view).await.unwrap() as u64;

    let view = AddUserToProjectQueryView::new(project_id, 2);
    assert!(db.execute(view).await.is_ok());

    let view = RemoveUserFromProjectQueryView::new(project_id, 2);
    let result = db.execute(view).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_remove_user_from_project_unknown_user() {
    let db = get_smart_db().await;

    let view = CreateProjectQueryView::new("Test Project", Some("Test Description"), 1);
    let project_id = db.fetch_scalar::<i32, _>(&view).await.unwrap() as u64;

    let view = RemoveUserFromProjectQueryView::new(project_id, 999);
    let result = db.execute(view).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_remove_user_from_project_unknown_project() {
    let db = get_smart_db().await;

    let view = RemoveUserFromProjectQueryView::new(999, 2);
    let result = db.execute(view).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_remove_user_from_project_unknown_project_and_unknown_user() {
    let db = get_smart_db().await;

    let view = RemoveUserFromProjectQueryView::new(999, 999);
    let result = db.execute(view).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
}
