use crate::common::fixtures::{create_user, set_task_status};
use crate::common::get_smart_db;
use mairie360_api_lib::smart_db::SmartDatabase;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use project_api::database::project::create::view::CreateProjectQueryView;
use project_api::database::tasks::collaboration::view::{
    AddTaskCommentQueryView, GetTaskCollaborationQueryView, TaskCollaborationRow, TaskComment,
};
use project_api::database::tasks::create_task::view::{
    CreateTaskQueryView, TaskPriority, TaskStatus,
};
use project_api::endpoints::v1::projects::project_id::tasks::task_id::collaboration::view::TaskCollaborationView;

async fn create_task(db: &SmartDatabase, owner: u64) -> (u64, u64) {
    let project_id = db
        .fetch_scalar::<i32, _>(&CreateProjectQueryView::new("Collaboration", None, owner))
        .await
        .unwrap() as u64;
    let task_id = db
        .fetch_scalar::<i32, _>(&CreateTaskQueryView::new(
            project_id,
            owner,
            "Tâche",
            "",
            TaskStatus::Todo,
            TaskPriority::Medium,
            None,
            None,
            &[],
        ))
        .await
        .unwrap() as u64;
    (project_id, task_id)
}

async fn collaboration(
    db: &SmartDatabase,
    project_id: u64,
    task_id: u64,
    limit: u32,
    offset: u32,
) -> TaskCollaborationView {
    let rows: Vec<TaskCollaborationRow> = db
        .fetch_all(&GetTaskCollaborationQueryView::new(
            project_id, task_id, limit, offset,
        ))
        .await
        .unwrap();
    rows.into_iter().next().expect("task found").into()
}

async fn comment(
    db: &SmartDatabase,
    project_id: u64,
    task_id: u64,
    author: u64,
    message: &str,
) -> Vec<TaskComment> {
    db.fetch_all(&AddTaskCommentQueryView::new(
        project_id, task_id, author, message,
    ))
    .await
    .unwrap()
}

#[tokio::test]
async fn test_comments_and_history_are_signed_and_read_back() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let author = create_user(&db, "Jeanne", None).await;
    let (project_id, task_id) = create_task(&db, author).await;

    let comments = comment(&db, project_id, task_id, author, "  Devis reçu.  ").await;
    set_task_status(&db, task_id, "in_progress").await;

    let created = &comments[0];
    assert!(created.id.starts_with("comment-"));
    assert_eq!(created.message, "Devis reçu.");
    assert_eq!(created.author.id, format!("user-{author}"));
    assert_eq!(created.author.name, "Jeanne Test");
    assert!(created.created_at.ends_with('Z'));

    let view = collaboration(&db, project_id, task_id, 100, 0).await;
    assert_eq!(view.comments, comments);
    assert_eq!(view.comments_total, 1);
    assert_eq!(
        view.history
            .iter()
            .map(|entry| entry.action.as_str())
            .collect::<Vec<_>>(),
        vec!["status_changed", "task_created"]
    );
    assert_eq!(view.history_total, 2);
    assert_eq!(view.history[0].label, "Status changed: todo → in_progress");
    assert_eq!(
        view.history[0].changes,
        Some(serde_json::json!({ "status": { "from": "todo", "to": "in_progress" } }))
    );
    assert_eq!(view.history[1].label, "Task created");
    assert_eq!(view.history[1].author.id, format!("user-{author}"));
    assert_eq!(view.history[1].changes, None);
}

#[tokio::test]
async fn test_comments_are_paginated_in_reading_order() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let author = create_user(&db, "Paul", None).await;
    let (project_id, task_id) = create_task(&db, author).await;
    for message in ["un", "deux", "trois"] {
        comment(&db, project_id, task_id, author, message).await;
    }

    let page = collaboration(&db, project_id, task_id, 2, 1).await;
    assert_eq!(page.comments_total, 3);
    assert_eq!(
        page.comments
            .iter()
            .map(|c| c.message.as_str())
            .collect::<Vec<_>>(),
        vec!["deux", "trois"]
    );
    assert_eq!(page.history_total, 1);
    assert!(page.history.is_empty());
}

#[tokio::test]
async fn test_collaboration_of_a_task_from_another_project_is_not_found() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let author = create_user(&db, "Claire", None).await;
    let (project_id, task_id) = create_task(&db, author).await;
    let other_project = project_id + 10_000;

    let rows: Vec<TaskCollaborationRow> = db
        .fetch_all(&GetTaskCollaborationQueryView::new(
            other_project,
            task_id,
            100,
            0,
        ))
        .await
        .unwrap();
    let comments = comment(&db, other_project, task_id, author, "Non").await;

    assert!(rows.is_empty());
    assert!(comments.is_empty());
    assert_eq!(
        collaboration(&db, project_id, task_id, 100, 0)
            .await
            .comments_total,
        0
    );
}

/// MAIR-502: `latest_comments_first` pages the comments from the most recent one, so page 1 holds
/// the latest comments, like the history; the totals do not change.
#[tokio::test]
async fn test_comments_can_be_paged_from_the_most_recent_one() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let author = create_user(&db, "Lina", None).await;
    let (project_id, task_id) = create_task(&db, author).await;
    for message in ["un", "deux", "trois"] {
        comment(&db, project_id, task_id, author, message).await;
    }
    let page = |offset| {
        GetTaskCollaborationQueryView::new(project_id, task_id, 2, offset).latest_comments_first()
    };
    let read = |rows: Vec<TaskCollaborationRow>| -> TaskCollaborationView {
        rows.into_iter().next().expect("task found").into()
    };

    let first = read(db.fetch_all(&page(0)).await.unwrap());
    assert_eq!(first.comments_total, 3);
    assert_eq!(
        first
            .comments
            .iter()
            .map(|c| c.message.as_str())
            .collect::<Vec<_>>(),
        vec!["trois", "deux"]
    );
    let second = read(db.fetch_all(&page(2)).await.unwrap());
    assert_eq!(
        second
            .comments
            .iter()
            .map(|c| c.message.as_str())
            .collect::<Vec<_>>(),
        vec!["un"]
    );
    assert_eq!(second.history_total, 1);
}
