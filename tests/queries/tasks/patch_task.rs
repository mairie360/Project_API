use crate::common::get_pool; // Utilisation de ta fonction utilitaire existante
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use project_api::database::project::create::query::create_project_query;
use project_api::database::project::create::view::CreateProjectQueryView;
use project_api::database::tasks::create_task::query::create_task_query;
use project_api::database::tasks::create_task::view::{
    CreateTaskQueryView, TaskPriority, TaskStatus,
};
use project_api::database::tasks::patch_task::query::patch_task_query;
use project_api::database::tasks::patch_task::view::PatchTaskQueryView;
use sqlx::types::Json;

#[sqlx::test]
async fn test_patch_task_name_success() {
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

    let view = CreateTaskQueryView::new(
        project_id,
        "Test Task",
        TaskStatus::Todo,
        TaskPriority::Medium,
        Some(chrono::Utc::now()),
        Some(1),
    );
    let result = create_task_query(view, pool.clone()).await;

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

    let view = PatchTaskQueryView::new(
        task_id as u64,
        Some("Updated Task"),
        None,
        None,
        None,
        None,
        None,
    );

    let result = patch_task_query(view, pool).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );

    let row_count = result.unwrap();
    assert_eq!(row_count, 1, "1 task should be updated, got: {}", row_count);
}

#[sqlx::test]
async fn test_patch_task_status_success() {
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

    let view = CreateTaskQueryView::new(
        project_id,
        "Test Task",
        TaskStatus::InProgress,
        TaskPriority::Medium,
        Some(chrono::Utc::now()),
        Some(1),
    );
    let result = create_task_query(view, pool.clone()).await;

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

    let view = PatchTaskQueryView::new(
        task_id as u64,
        None,
        Some(TaskStatus::InProgress),
        None,
        None,
        None,
        None,
    );

    let result = patch_task_query(view, pool).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );

    let row_count = result.unwrap();
    assert_eq!(row_count, 1, "1 task should be updated, got: {}", row_count);
}

#[sqlx::test]
async fn test_patch_task_priority_success() {
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

    let view = CreateTaskQueryView::new(
        project_id,
        "Test Task",
        TaskStatus::Completed,
        TaskPriority::Medium,
        Some(chrono::Utc::now()),
        Some(1),
    );
    let result = create_task_query(view, pool.clone()).await;

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

    let view = PatchTaskQueryView::new(
        task_id as u64,
        None,
        None,
        Some(TaskPriority::High),
        None,
        None,
        None,
    );

    let result = patch_task_query(view, pool).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );

    let row_count = result.unwrap();
    assert_eq!(row_count, 1, "1 task should be updated, got: {}", row_count);
}
#[sqlx::test]
async fn test_patch_task_assigned_success() {
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

    let view = CreateTaskQueryView::new(
        project_id,
        "Test Task",
        TaskStatus::Completed,
        TaskPriority::Low,
        Some(chrono::Utc::now()),
        Some(1),
    );
    let result = create_task_query(view, pool.clone()).await;

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

    let view = PatchTaskQueryView::new(task_id as u64, None, None, None, None, Some(2), None);

    let result = patch_task_query(view, pool).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );

    let row_count = result.unwrap();
    assert_eq!(row_count, 1, "1 task should be updated, got: {}", row_count);
}

#[sqlx::test]
async fn test_patch_task_fields_success() {
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

    let view = CreateTaskQueryView::new(
        project_id,
        "Test Task",
        TaskStatus::Completed,
        TaskPriority::High,
        Some(chrono::Utc::now()),
        Some(1),
    );
    let result = create_task_query(view, pool.clone()).await;

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

    let view = PatchTaskQueryView::new(
        task_id as u64,
        None,
        None,
        None,
        None,
        None,
        Some(Json(vec![])),
    );

    let result = patch_task_query(view, pool).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );

    let row_count = result.unwrap();
    assert_eq!(row_count, 1, "1 task should be updated, got: {}", row_count);
}

#[sqlx::test]
async fn test_patch_task_unknown_task() {
    let (_container, host) = get_shared_db().await;
    let pool = get_pool(host.to_string()).await;

    let view = PatchTaskQueryView::new(999, None, None, None, None, None, None);

    let result = patch_task_query(view, pool).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
    let row_count = result.unwrap();
    assert_eq!(
        row_count, 0,
        "0 tasks should be updated, got: {}",
        row_count
    );
}
