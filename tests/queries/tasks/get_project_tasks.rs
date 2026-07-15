use crate::common::get_pool; // Utilisation de ta fonction utilitaire existante
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use project_api::database::project::create::query::create_project_query;
use project_api::database::project::create::view::CreateProjectQueryView;
use project_api::database::tasks::create_task::query::create_task_query;
use project_api::database::tasks::create_task::view::{
    CreateTaskQueryView, TaskPriority, TaskStatus,
};
use project_api::database::tasks::get_project_tasks::query::get_project_tasks_query;
use project_api::database::tasks::get_project_tasks::view::GetProjectTasksQueryView;

#[sqlx::test]
async fn test_get_tasks_success() {
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

    let view = CreateTaskQueryView::new(
        project_id,
        "Test Task",
        TaskStatus::Completed,
        TaskPriority::High,
        None,
        Some(1),
    );
    let result = create_task_query(view, pool.clone()).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );

    let view = CreateTaskQueryView::new(
        project_id,
        "Test Task",
        TaskStatus::Completed,
        TaskPriority::High,
        Some(chrono::Utc::now()),
        None,
    );
    let result = create_task_query(view, pool.clone()).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );

    let view = CreateTaskQueryView::new(
        project_id,
        "Test Task",
        TaskStatus::Completed,
        TaskPriority::High,
        Some(chrono::Utc::now()),
        None,
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

    let view = GetProjectTasksQueryView::new(project_id);
    let result = get_project_tasks_query(view, pool.clone()).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );

    let tasks = result.unwrap();
    assert_eq!(
        tasks.len(),
        8,
        "Expected tasks to have length 1, got: {}",
        tasks.len()
    );
}

#[sqlx::test]
async fn test_get_task_unknown_project() {
    let (_container, host) = get_shared_db().await;
    let pool = get_pool(host.to_string()).await;

    let view = GetProjectTasksQueryView::new(999);
    let result = get_project_tasks_query(view, pool).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );

    let tasks = result.unwrap();
    assert_eq!(
        tasks.len(),
        0,
        "Expected tasks to have length 0, got: {}",
        tasks.len()
    );
}
