use crate::common::get_pool; // Utilisation de ta fonction utilitaire existante
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use project_api::database::project::create::query::create_project_query;
use project_api::database::project::create::view::CreateProjectQueryView;
use project_api::database::tasks::create_task::query::create_task_query;
use project_api::database::tasks::create_task::view::{
    CreateTaskQueryView, TaskPriority, TaskStatus,
};
use project_api::database::tasks::delete_task::query::delete_task_query;
use project_api::database::tasks::delete_task::view::DeleteTaskQueryView;

#[sqlx::test]
async fn test_delete_task_success() {
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

    let task_id = result.unwrap() as u64;

    let view = DeleteTaskQueryView::new(task_id);
    let result = delete_task_query(view, pool).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );

    let rows_affected = result.unwrap();
    assert!(
        rows_affected == 1,
        "Expected 1 row affected, got: {}",
        rows_affected
    );
}

#[sqlx::test]
async fn test_delete_task_unknown_task() {
    let (_container, host) = get_shared_db().await;
    let pool = get_pool(host.to_string()).await;

    let view = DeleteTaskQueryView::new(999);
    let result = delete_task_query(view, pool).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );

    let rows_affected = result.unwrap();
    assert!(
        rows_affected == 0,
        "Expected 0 row affected, got: {}",
        rows_affected
    );
}
