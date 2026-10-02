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
    CreateTaskQueryView, TaskPriority, TaskStatus,
};
use project_api::database::tasks::get_project_tasks::view::{
    DynamicTaskField, FieldType, GetProjectTasksQueryView, Task,
};
use project_api::database::tasks::patch_task::view::{PatchTaskQueryView, TaskChanges};
use project_api::database::users::add_user_to_project::view::AddUserToProjectQueryView;

async fn create_task(db: &SmartDatabase) -> (u64, u64) {
    let view = CreateProjectQueryView::new("Test Project", Some("Test Description"), 1);
    let project_id = db.fetch_scalar::<i32, _>(&view).await.unwrap() as u64;

    let view = CreateTaskQueryView::new(
        project_id,
        1,
        "Test Task",
        "Initial description",
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
    let tasks: PagedRows<Task> = db
        .fetch_one(&GetProjectTasksQueryView::new(project_id, 100, 0))
        .await
        .unwrap();
    tasks.items.into_iter().next().expect("task created")
}

async fn patch(
    db: &SmartDatabase,
    project_id: u64,
    task_id: u64,
    by: u64,
    changes: TaskChanges<'_>,
) -> bool {
    db.fetch_scalar(&PatchTaskQueryView::new(project_id, task_id, by, changes))
        .await
        .unwrap()
}

#[tokio::test]
async fn test_patch_task_updates_only_provided_fields() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let (project_id, task_id) = create_task(&db).await;
    let due = chrono::DateTime::parse_from_rfc3339("2026-11-02T08:30:00Z")
        .unwrap()
        .with_timezone(&chrono::Utc);

    let updated = patch(
        &db,
        project_id,
        task_id,
        1,
        TaskChanges {
            title: Some("Updated Task"),
            status: Some(TaskStatus::InProgress),
            priority: Some(TaskPriority::High),
            due_date: Some(due),
            ..TaskChanges::default()
        },
    )
    .await;

    assert!(updated);
    let task = read_task(&db, project_id).await;
    assert_eq!(task.title(), "Updated Task");
    assert_eq!(task.description(), "Initial description");
    assert_eq!(task.status(), "in_progress");
    assert_eq!(task.priority(), "high");
    assert_eq!(task.due_date(), Some(due));
    assert_eq!(task.assigned_to(), Some(1));
}

#[tokio::test]
async fn test_patch_task_persists_description_and_fields() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let (project_id, task_id) = create_task(&db).await;
    let fields = [DynamicTaskField {
        label: "Date de la réunion".to_string(),
        task_type: FieldType::Date,
        fields_options: vec![],
    }];

    assert!(
        patch(
            &db,
            project_id,
            task_id,
            1,
            TaskChanges {
                description: Some("Réunion publique le 3 octobre"),
                fields: Some(&fields),
                ..TaskChanges::default()
            },
        )
        .await
    );
    let task = read_task(&db, project_id).await;
    assert_eq!(task.description(), "Réunion publique le 3 octobre");
    assert_eq!(task.fields(), fields.to_vec());

    // An empty description clears it; absent fields are kept.
    assert!(
        patch(
            &db,
            project_id,
            task_id,
            1,
            TaskChanges {
                description: Some(""),
                ..TaskChanges::default()
            },
        )
        .await
    );
    let task = read_task(&db, project_id).await;
    assert_eq!(task.description(), "");
    assert_eq!(task.fields(), fields.to_vec());
}

#[tokio::test]
async fn test_patch_task_is_logged_and_signed_by_the_caller() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let editor = create_user(&db, "Hugo", None).await;
    let (project_id, task_id) = create_task(&db).await;

    assert!(
        patch(
            &db,
            project_id,
            task_id,
            editor,
            TaskChanges {
                title: Some("Renamed"),
                status: Some(TaskStatus::Completed),
                ..TaskChanges::default()
            },
        )
        .await
    );

    let rows: Vec<TaskCollaborationRow> = db
        .fetch_all(&GetTaskCollaborationQueryView::new(
            project_id, task_id, 100, 0,
        ))
        .await
        .unwrap();
    let history = &rows[0].history;
    assert_eq!(history.total, 3);
    let mut actions: Vec<&str> = history.items.iter().map(|h| h.action.as_str()).collect();
    actions.sort_unstable();
    assert_eq!(
        actions,
        vec!["status_changed", "task_created", "task_updated"]
    );
    for entry in history.items.iter().filter(|h| h.action != "task_created") {
        assert_eq!(entry.author_id, Some(editor as i32));
    }
    let updated = history
        .items
        .iter()
        .find(|h| h.action == "task_updated")
        .unwrap();
    assert_eq!(
        updated.changes,
        Some(serde_json::json!({ "title": { "from": "Test Task", "to": "Renamed" } }))
    );
}

#[tokio::test]
async fn test_patch_task_assignee_must_belong_to_the_project() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let (project_id, task_id) = create_task(&db).await;
    let outsider = create_user(&db, "Outsider", None).await;
    let member = create_user(&db, "Member", None).await;
    db.execute(AddUserToProjectQueryView::new(project_id, member))
        .await
        .unwrap();

    let refused = patch(
        &db,
        project_id,
        task_id,
        1,
        TaskChanges {
            title: Some("Not applied"),
            assigned_to: Some(Some(outsider)),
            ..TaskChanges::default()
        },
    )
    .await;
    assert!(!refused);
    let task = read_task(&db, project_id).await;
    assert_eq!(task.assigned_to(), Some(1));
    assert_eq!(task.title(), "Test Task");

    let assigned = patch(
        &db,
        project_id,
        task_id,
        1,
        TaskChanges {
            assigned_to: Some(Some(member)),
            ..TaskChanges::default()
        },
    )
    .await;
    assert!(assigned);
    assert_eq!(
        read_task(&db, project_id).await.assigned_to(),
        Some(member as i32)
    );

    let cleared = patch(
        &db,
        project_id,
        task_id,
        1,
        TaskChanges {
            assigned_to: Some(None),
            ..TaskChanges::default()
        },
    )
    .await;
    assert!(cleared);
    assert_eq!(read_task(&db, project_id).await.assigned_to(), None);
}

#[tokio::test]
async fn test_patch_task_of_another_project_is_not_found() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let (project_id, task_id) = create_task(&db).await;
    let rename = || TaskChanges {
        title: Some("Updated Task"),
        ..TaskChanges::default()
    };

    let wrong_project = patch(&db, project_id + 10_000, task_id, 1, rename()).await;
    let unknown_task = patch(&db, project_id, 999_999, 1, rename()).await;

    assert!(!wrong_project);
    assert!(!unknown_task);
    assert_eq!(read_task(&db, project_id).await.title(), "Test Task");
}
