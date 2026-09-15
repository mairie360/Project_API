use crate::common::get_smart_db;
use mairie360_api_lib::smart_db::SmartDatabase;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use project_api::database::project::create::view::CreateProjectQueryView;
use project_api::database::tasks::create_task::view::{
    CreateTaskQueryView, TaskPriority, TaskStatus,
};
use project_api::database::tasks::get_project_tasks::view::{GetProjectTasksQueryView, Task};
use project_api::database::tasks::patch_task::view::PatchTaskQueryView;

async fn create_task(db: &SmartDatabase) -> (u64, u64) {
    let view = CreateProjectQueryView::new("Test Project", Some("Test Description"), 1);
    let project_id = db.fetch_scalar::<i32, _>(&view).await.unwrap() as u64;

    let view = CreateTaskQueryView::new(
        project_id,
        "Test Task",
        TaskStatus::Todo,
        TaskPriority::Medium,
        Some(chrono::Utc::now()),
        Some(1),
        &[],
    );
    let task_id = db.fetch_scalar::<i32, _>(&view).await.unwrap() as u64;
    (project_id, task_id)
}

async fn read_task(db: &SmartDatabase, project_id: u64) -> Task {
    let tasks: Vec<Task> = db
        .fetch_all(&GetProjectTasksQueryView::new(project_id))
        .await
        .unwrap();
    tasks.into_iter().next().expect("tâche créée")
}

#[tokio::test]
async fn test_patch_task_updates_only_provided_fields() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let (project_id, task_id) = create_task(&db).await;
    let due = chrono::DateTime::parse_from_rfc3339("2026-11-02T08:30:00Z")
        .unwrap()
        .with_timezone(&chrono::Utc);

    let updated: bool = db
        .fetch_scalar(&PatchTaskQueryView::new(
            project_id,
            task_id,
            Some("Updated Task"),
            Some(TaskStatus::InProgress),
            Some(TaskPriority::High),
            Some(due),
            None,
        ))
        .await
        .unwrap();

    assert!(updated);
    let task = read_task(&db, project_id).await;
    assert_eq!(task.title(), "Updated Task");
    assert_eq!(task.status(), "in_progress");
    assert_eq!(task.priority(), "high");
    assert_eq!(task.due_date(), Some(due));
    assert_eq!(task.assigned_to(), Some(1));
}

#[tokio::test]
async fn test_patch_task_sets_and_clears_the_assignee() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let (project_id, task_id) = create_task(&db).await;

    let assigned: bool = db
        .fetch_scalar(&PatchTaskQueryView::new(
            project_id,
            task_id,
            None,
            None,
            None,
            None,
            Some(Some(2)),
        ))
        .await
        .unwrap();
    assert!(assigned);
    assert_eq!(read_task(&db, project_id).await.assigned_to(), Some(2));

    let cleared: bool = db
        .fetch_scalar(&PatchTaskQueryView::new(
            project_id,
            task_id,
            None,
            None,
            None,
            None,
            Some(None),
        ))
        .await
        .unwrap();
    assert!(cleared);
    let task = read_task(&db, project_id).await;
    assert_eq!(task.assigned_to(), None);
    assert_eq!(task.title(), "Test Task");
}

#[tokio::test]
async fn test_patch_task_of_another_project_is_not_found() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let (project_id, task_id) = create_task(&db).await;

    let wrong_project: bool = db
        .fetch_scalar(&PatchTaskQueryView::new(
            project_id + 10_000,
            task_id,
            Some("Updated Task"),
            None,
            None,
            None,
            None,
        ))
        .await
        .unwrap();
    let unknown_task: bool = db
        .fetch_scalar(&PatchTaskQueryView::new(
            project_id,
            999_999,
            Some("Updated Task"),
            None,
            None,
            None,
            None,
        ))
        .await
        .unwrap();

    assert!(!wrong_project);
    assert!(!unknown_task);
    assert_eq!(read_task(&db, project_id).await.title(), "Test Task");
}
