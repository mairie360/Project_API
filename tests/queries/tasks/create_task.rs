use crate::common::get_smart_db;
use mairie360_api_lib::smart_db::SmartDatabase;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use project_api::database::project::create::view::CreateProjectQueryView;
use project_api::database::tasks::create_task::view::{
    CreateTaskQueryView, TaskPriority, TaskStatus,
};

async fn create_project(db: &SmartDatabase) -> u64 {
    let view = CreateProjectQueryView::new("Test Project", Some("Test Description"), 1);
    let result = db.fetch_scalar::<i32, _>(&view).await;
    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
    result.unwrap() as u64
}

async fn assert_task_created(
    db: &SmartDatabase,
    project_id: u64,
    status: TaskStatus,
    priority: TaskPriority,
    due_date: Option<chrono::DateTime<chrono::Utc>>,
    assigned_to: Option<u64>,
) {
    let view = CreateTaskQueryView::new(
        project_id,
        "Test Task",
        status,
        priority,
        due_date,
        assigned_to,
    );
    let result = db.fetch_scalar::<i32, _>(&view).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
    let task_id = result.unwrap();
    assert!(
        task_id != 0,
        "Expected task_id to be non-zero, got: {}",
        task_id
    );
}

#[tokio::test]
async fn test_create_task_todo_success() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let project_id = create_project(&db).await;

    assert_task_created(
        &db,
        project_id,
        TaskStatus::Todo,
        TaskPriority::Medium,
        Some(chrono::Utc::now()),
        Some(1),
    )
    .await;
}

#[tokio::test]
async fn test_create_task_in_progress_success() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let project_id = create_project(&db).await;

    assert_task_created(
        &db,
        project_id,
        TaskStatus::InProgress,
        TaskPriority::Medium,
        Some(chrono::Utc::now()),
        Some(1),
    )
    .await;
}

#[tokio::test]
async fn test_create_task_completed_success() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let project_id = create_project(&db).await;

    assert_task_created(
        &db,
        project_id,
        TaskStatus::Completed,
        TaskPriority::Medium,
        Some(chrono::Utc::now()),
        Some(1),
    )
    .await;
}

#[tokio::test]
async fn test_create_task_status_error_error() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let project_id = create_project(&db).await;

    let view = CreateTaskQueryView::new(
        project_id,
        "Test Task",
        TaskStatus::Error,
        TaskPriority::Medium,
        Some(chrono::Utc::now()),
        Some(1),
    );
    let result = db.fetch_scalar::<i32, _>(&view).await;

    assert!(
        result.is_err(),
        "Expected result to be Err, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_create_task_low_priority_success() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let project_id = create_project(&db).await;

    assert_task_created(
        &db,
        project_id,
        TaskStatus::Completed,
        TaskPriority::Low,
        Some(chrono::Utc::now()),
        Some(1),
    )
    .await;
}

#[tokio::test]
async fn test_create_task_high_priority_success() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let project_id = create_project(&db).await;

    assert_task_created(
        &db,
        project_id,
        TaskStatus::Completed,
        TaskPriority::High,
        Some(chrono::Utc::now()),
        Some(1),
    )
    .await;
}

#[tokio::test]
async fn test_create_task_no_due_date_success() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let project_id = create_project(&db).await;

    assert_task_created(
        &db,
        project_id,
        TaskStatus::Completed,
        TaskPriority::High,
        None,
        Some(1),
    )
    .await;
}

#[tokio::test]
async fn test_create_task_no_owner_success() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let project_id = create_project(&db).await;

    assert_task_created(
        &db,
        project_id,
        TaskStatus::Completed,
        TaskPriority::High,
        Some(chrono::Utc::now()),
        None,
    )
    .await;
}

#[tokio::test]
async fn test_create_task_no_owner_and_no_due_date_success() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let project_id = create_project(&db).await;

    assert_task_created(
        &db,
        project_id,
        TaskStatus::Completed,
        TaskPriority::High,
        None,
        None,
    )
    .await;
}

#[tokio::test]
async fn test_create_task_unknown_project() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;

    let view = CreateTaskQueryView::new(
        999,
        "Test Task",
        TaskStatus::Completed,
        TaskPriority::High,
        Some(chrono::Utc::now()),
        Some(1),
    );
    let result = db.fetch_scalar::<i32, _>(&view).await;

    assert!(
        result.is_err(),
        "Expected result to be Err, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_create_task_unknown_owner() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let project_id = create_project(&db).await;

    let view = CreateTaskQueryView::new(
        project_id,
        "Test Task",
        TaskStatus::Completed,
        TaskPriority::High,
        Some(chrono::Utc::now()),
        Some(999),
    );
    let result = db.fetch_scalar::<i32, _>(&view).await;

    assert!(
        result.is_err(),
        "Expected result to be Err, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_create_task_unknown_project_and_owner() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;

    let view = CreateTaskQueryView::new(
        999,
        "Test Task",
        TaskStatus::Completed,
        TaskPriority::High,
        Some(chrono::Utc::now()),
        Some(999),
    );
    let result = db.fetch_scalar::<i32, _>(&view).await;

    assert!(
        result.is_err(),
        "Expected result to be Err, got: {:?}",
        result
    );
}
