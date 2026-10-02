use crate::common::fixtures::create_user;
use crate::common::get_smart_db;
use mairie360_api_lib::smart_db::SmartDatabase;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use project_api::database::paged::PagedRows;
use project_api::database::project::create::view::CreateProjectQueryView;
use project_api::database::tasks::collaboration::view::{
    GetTaskCollaborationQueryView, TaskCollaborationRow,
};
use project_api::database::tasks::create_task::view::{
    CreateTaskQueryView, TaskPriority, TaskStatus, ASSIGNEE_NOT_IN_PROJECT,
};
use project_api::database::tasks::get_project_tasks::view::{GetProjectTasksQueryView, Task};
use project_api::database::users::add_user_to_project::view::AddUserToProjectQueryView;

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
        1,
        "Test Task",
        "",
        status,
        priority,
        due_date,
        assigned_to,
        &[],
    );
    let result = db.fetch_scalar::<i32, _>(&view).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
    let task_id = result.unwrap();
    assert!(
        task_id != ASSIGNEE_NOT_IN_PROJECT,
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
        1,
        "Test Task",
        "",
        TaskStatus::Error,
        TaskPriority::Medium,
        Some(chrono::Utc::now()),
        Some(1),
        &[],
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
        999_999,
        1,
        "Test Task",
        "",
        TaskStatus::Completed,
        TaskPriority::High,
        Some(chrono::Utc::now()),
        None,
        &[],
    );
    let result = db.fetch_scalar::<i32, _>(&view).await;

    assert!(
        result.is_err(),
        "Expected result to be Err, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_create_task_assignee_outside_the_project_is_refused() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let project_id = create_project(&db).await;
    let outsider = create_user(&db, "Outsider", None).await;

    for assignee in [outsider, 999_999] {
        let view = CreateTaskQueryView::new(
            project_id,
            1,
            "Test Task",
            "",
            TaskStatus::Todo,
            TaskPriority::High,
            None,
            Some(assignee),
            &[],
        );
        let result = db.fetch_scalar::<i32, _>(&view).await.unwrap();
        assert_eq!(result, ASSIGNEE_NOT_IN_PROJECT);
    }
    let tasks: PagedRows<Task> = db
        .fetch_one(&GetProjectTasksQueryView::new(project_id, 100, 0))
        .await
        .unwrap();
    assert_eq!(tasks.total, 0);
}

#[tokio::test]
async fn test_create_task_assignee_member_of_the_project_is_accepted() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let project_id = create_project(&db).await;
    let member = create_user(&db, "Member", None).await;
    db.execute(AddUserToProjectQueryView::new(project_id, member))
        .await
        .unwrap();

    assert_task_created(
        &db,
        project_id,
        TaskStatus::Todo,
        TaskPriority::Low,
        None,
        Some(member),
    )
    .await;
}

#[tokio::test]
async fn test_create_task_persists_the_description_and_logs_the_creation() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let project_id = create_project(&db).await;
    let author = create_user(&db, "Creator", None).await;

    let task_id = db
        .fetch_scalar::<i32, _>(&CreateTaskQueryView::new(
            project_id,
            author,
            "Consulter les riverains",
            "Réunion publique à organiser avant le 15 octobre",
            TaskStatus::Todo,
            TaskPriority::Medium,
            None,
            None,
            &[],
        ))
        .await
        .unwrap() as u64;

    let tasks: PagedRows<Task> = db
        .fetch_one(&GetProjectTasksQueryView::new(project_id, 100, 0))
        .await
        .unwrap();
    assert_eq!(
        tasks.items[0].description(),
        "Réunion publique à organiser avant le 15 octobre"
    );

    let rows: Vec<TaskCollaborationRow> = db
        .fetch_all(&GetTaskCollaborationQueryView::new(
            project_id, task_id, 100, 0,
        ))
        .await
        .unwrap();
    let history = &rows[0].history.items;
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].action, "task_created");
    assert_eq!(history[0].author_id, Some(author as i32));
}
