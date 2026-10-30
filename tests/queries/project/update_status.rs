use crate::common::get_smart_db;
use project_api::database::project::create::view::CreateProjectQueryView;
use project_api::database::project::update_status::view::{
    ProjectStatus, UpdateProjectStatusQueryView,
};

async fn create_project(db: &mairie360_api_lib::smart_db::SmartDatabase) -> u64 {
    let view = CreateProjectQueryView::new("Test Project", Some("Test Description"), 1);
    let result = db.fetch_scalar::<i32, _>(&view).await;
    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
    result.unwrap() as u64
}

#[tokio::test]
async fn test_update_project_status_to_suspended_success() {
    let db = get_smart_db().await;
    let project_id = create_project(&db).await;

    let view = UpdateProjectStatusQueryView::new(project_id, ProjectStatus::Suspended);
    let result = db.execute(view).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_update_project_status_to_active_success() {
    let db = get_smart_db().await;
    let project_id = create_project(&db).await;

    let view = UpdateProjectStatusQueryView::new(project_id, ProjectStatus::Active);
    let result = db.execute(view).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_update_project_status_to_completed_success() {
    let db = get_smart_db().await;
    let project_id = create_project(&db).await;

    let view = UpdateProjectStatusQueryView::new(project_id, ProjectStatus::Completed);
    let result = db.execute(view).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_update_project_status_unknown_status_error() {
    let db = get_smart_db().await;
    let project_id = create_project(&db).await;

    let view = UpdateProjectStatusQueryView::new(project_id, ProjectStatus::Error);
    let result = db.execute(view).await;

    assert!(
        result.is_err(),
        "Expected result to be Err, got: {:?}",
        result
    );
}
