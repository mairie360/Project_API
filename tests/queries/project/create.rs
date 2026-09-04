use crate::common::get_smart_db;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use project_api::database::project::create::view::CreateProjectQueryView;

#[tokio::test]
async fn test_create_project_success() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;

    let view = CreateProjectQueryView::new("Test Project", Some("Test Description"), 1);

    let result = db.fetch_scalar::<i32, _>(&view).await;

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
}

#[tokio::test]
async fn test_create_project_none_description_success() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;

    let view = CreateProjectQueryView::new("Test Project", None, 1);

    let result = db.fetch_scalar::<i32, _>(&view).await;

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
}

#[tokio::test]
async fn test_create_project_unknown_owner() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;

    let view = CreateProjectQueryView::new("Test Project", Some("Test Description"), 999);

    let result = db.fetch_scalar::<i32, _>(&view).await;

    assert!(
        result.is_err(),
        "Expected result to be Err, got: {:?}",
        result
    );
}
