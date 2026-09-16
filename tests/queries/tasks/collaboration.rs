use crate::common::fixtures::{create_user, set_task_status};
use crate::common::get_smart_db;
use mairie360_api_lib::smart_db::SmartDatabase;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use project_api::database::project::create::view::CreateProjectQueryView;
use project_api::database::tasks::collaboration::view::{
    AddTaskCommentQueryView, AppendTaskHistoryQueryView, GetTaskCollaborationQueryView,
    TaskCollaborationRow, TaskComment, TaskHistoryEntry,
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
            "Tâche",
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

async fn collaboration(db: &SmartDatabase, project_id: u64, task_id: u64) -> TaskCollaborationView {
    let rows: Vec<TaskCollaborationRow> = db
        .fetch_all(&GetTaskCollaborationQueryView::new(project_id, task_id))
        .await
        .unwrap();
    rows.into_iter().next().expect("tâche trouvée").into()
}

#[tokio::test]
async fn test_comments_and_history_are_signed_and_read_back() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let author = create_user(&db, "Jeanne", None).await;
    let (project_id, task_id) = create_task(&db, author).await;

    let comments: Vec<TaskComment> = db
        .fetch_all(&AddTaskCommentQueryView::new(
            project_id,
            task_id,
            author,
            "  Devis reçu.  ",
        ))
        .await
        .unwrap();
    let entries: Vec<TaskHistoryEntry> = db
        .fetch_all(&AppendTaskHistoryQueryView::new(
            project_id,
            task_id,
            author,
            "task_updated",
            "Tâche modifiée.",
            Some(&serde_json::json!({ "title": "Tâche" })),
        ))
        .await
        .unwrap();
    set_task_status(&db, task_id, "in_progress").await;

    let comment = &comments[0];
    assert!(comment.id.starts_with("comment-"));
    assert_eq!(comment.message, "Devis reçu.");
    assert_eq!(comment.author.id, format!("user-{author}"));
    assert_eq!(comment.author.name, "Jeanne Test");
    assert!(comment.created_at.ends_with('Z'));
    assert_eq!(
        entries[0].changes,
        Some(serde_json::json!({ "title": "Tâche" }))
    );

    let view = collaboration(&db, project_id, task_id).await;
    assert_eq!(view.comments, comments);
    assert_eq!(
        view.history
            .iter()
            .map(|entry| entry.action.as_str())
            .collect::<Vec<_>>(),
        vec!["status_changed", "task_updated"]
    );
    assert_eq!(view.history[0].label, "Statut modifié : todo → in_progress");
    assert_eq!(
        view.history[0].changes,
        Some(serde_json::json!({ "status": { "from": "todo", "to": "in_progress" } }))
    );
}

#[tokio::test]
async fn test_history_without_changes_omits_the_field() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let author = create_user(&db, "Paul", None).await;
    let (project_id, task_id) = create_task(&db, author).await;

    let entries: Vec<TaskHistoryEntry> = db
        .fetch_all(&AppendTaskHistoryQueryView::new(
            project_id,
            task_id,
            author,
            "task_created",
            "Tâche créée.",
            None,
        ))
        .await
        .unwrap();

    assert_eq!(entries[0].changes, None);
}

#[tokio::test]
async fn test_collaboration_of_a_task_from_another_project_is_not_found() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let author = create_user(&db, "Claire", None).await;
    let (project_id, task_id) = create_task(&db, author).await;
    let other_project = project_id + 10_000;

    let rows: Vec<TaskCollaborationRow> = db
        .fetch_all(&GetTaskCollaborationQueryView::new(other_project, task_id))
        .await
        .unwrap();
    let comments: Vec<TaskComment> = db
        .fetch_all(&AddTaskCommentQueryView::new(
            other_project,
            task_id,
            author,
            "Non",
        ))
        .await
        .unwrap();

    assert!(rows.is_empty());
    assert!(comments.is_empty());
    assert!(collaboration(&db, project_id, task_id)
        .await
        .comments
        .is_empty());
}
