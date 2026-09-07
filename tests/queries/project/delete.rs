use crate::common::get_smart_db;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use project_api::database::project::create::view::CreateProjectQueryView;
use project_api::database::project::delete::view::DeleteProjectQueryView;

#[tokio::test]
async fn test_delete_project_success() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;

    let view = CreateProjectQueryView::new("Test Project", Some("Test Description"), 1);
    let result = db.fetch_scalar::<i32, _>(&view).await;
    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
    let project_id = result.unwrap() as u64;

    let view = DeleteProjectQueryView::new(project_id);
    let result = db.execute(view).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_delete_project_unknown_project() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;

    let view = DeleteProjectQueryView::new(999);
    let result = db.execute(view).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
}
