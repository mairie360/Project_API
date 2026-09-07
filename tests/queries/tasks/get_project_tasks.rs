use crate::common::get_smart_db;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use project_api::database::project::create::view::CreateProjectQueryView;
use project_api::database::tasks::create_task::view::{
    CreateTaskQueryView, TaskPriority, TaskStatus,
};
use project_api::database::tasks::get_project_tasks::view::{GetProjectTasksQueryView, Task};

#[tokio::test]
async fn test_get_tasks_success() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;

    let view = CreateProjectQueryView::new("Test Project", Some("Test Description"), 1);
    let project_id = db.fetch_scalar::<i32, _>(&view).await.unwrap() as u64;

    let now = chrono::Utc::now();
    let specs = [
        (
            TaskStatus::Todo,
            TaskPriority::Medium,
            Some(now),
            Some(1u64),
        ),
        (
            TaskStatus::InProgress,
            TaskPriority::Medium,
            Some(now),
            Some(1),
        ),
        (
            TaskStatus::Completed,
            TaskPriority::Medium,
            Some(now),
            Some(1),
        ),
        (TaskStatus::Completed, TaskPriority::Low, Some(now), Some(1)),
        (
            TaskStatus::Completed,
            TaskPriority::High,
            Some(now),
            Some(1),
        ),
        (TaskStatus::Completed, TaskPriority::High, None, Some(1)),
        (TaskStatus::Completed, TaskPriority::High, Some(now), None),
        (TaskStatus::Completed, TaskPriority::High, Some(now), None),
    ];

    for (status, priority, due_date, assigned_to) in specs {
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
    }

    let view = GetProjectTasksQueryView::new(project_id);
    let result = db.fetch_all::<Task, _>(&view).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
    let tasks = result.unwrap();
    assert_eq!(
        tasks.len(),
        8,
        "Expected tasks to have length 8, got: {}",
        tasks.len()
    );
}

#[tokio::test]
async fn test_get_task_unknown_project() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;

    let view = GetProjectTasksQueryView::new(999);
    let result = db.fetch_all::<Task, _>(&view).await;

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
