use crate::common::get_smart_db;
use mairie360_api_lib::smart_db::SmartDatabase;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use project_api::database::project::create::view::CreateProjectQueryView;
use project_api::database::tasks::create_task::view::{
    CreateTaskQueryView, TaskPriority, TaskStatus,
};
use project_api::database::tasks::patch_task::view::PatchTaskQueryView;

async fn create_task(db: &SmartDatabase, status: TaskStatus, priority: TaskPriority) -> u64 {
    let view = CreateProjectQueryView::new("Test Project", Some("Test Description"), 1);
    let project_id = db.fetch_scalar::<i32, _>(&view).await.unwrap() as u64;

    let view = CreateTaskQueryView::new(
        project_id,
        "Test Task",
        status,
        priority,
        Some(chrono::Utc::now()),
        Some(1),
    );
    let result = db.fetch_scalar::<i32, _>(&view).await;
    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
    result.unwrap() as u64
}

#[tokio::test]
async fn test_patch_task_name_success() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let task_id = create_task(&db, TaskStatus::Todo, TaskPriority::Medium).await;

    let view = PatchTaskQueryView::new(task_id, Some("Updated Task"), None, None, None, None, None);
    let result = db.execute(view).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_patch_task_status_success() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let task_id = create_task(&db, TaskStatus::InProgress, TaskPriority::Medium).await;

    let view = PatchTaskQueryView::new(
        task_id,
        None,
        Some(TaskStatus::InProgress),
        None,
        None,
        None,
        None,
    );
    let result = db.execute(view).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_patch_task_priority_success() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let task_id = create_task(&db, TaskStatus::Completed, TaskPriority::Medium).await;

    let view = PatchTaskQueryView::new(
        task_id,
        None,
        None,
        Some(TaskPriority::High),
        None,
        None,
        None,
    );
    let result = db.execute(view).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_patch_task_assigned_success() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let task_id = create_task(&db, TaskStatus::Completed, TaskPriority::Low).await;

    let view = PatchTaskQueryView::new(task_id, None, None, None, None, Some(2), None);
    let result = db.execute(view).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_patch_task_unknown_task() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;

    let view = PatchTaskQueryView::new(999, Some("Updated Task"), None, None, None, None, None);
    let result = db.execute(view).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
}
