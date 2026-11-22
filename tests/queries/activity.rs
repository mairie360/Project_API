//! MAIR-1089: the activity feed of a project through the query views and the `project_api` role.

use crate::common::fixtures::{create_user, set_task_status};
use crate::common::get_smart_db;
use project_api::database::activity::view::{
    ActivityRow, AddProjectCommentQueryView, GetProjectActivityQueryView,
};
use project_api::database::paged::PagedRows;
use project_api::database::project::create::view::CreateProjectQueryView;
use project_api::database::tasks::create_task::view::{
    CreateTaskQueryView, TaskPriority, TaskStatus,
};

#[tokio::test]
async fn mair_1089_the_feed_mixes_comments_and_task_history_newest_first() {
    let db = get_smart_db().await;
    let owner = create_user("Owner", None).await;
    let project = db
        .fetch_scalar::<i32, _>(&CreateProjectQueryView::new("Médiathèque", None, owner))
        .await
        .unwrap() as u64;
    let other = db
        .fetch_scalar::<i32, _>(&CreateProjectQueryView::new("Voirie", None, owner))
        .await
        .unwrap() as u64;
    let task = db
        .fetch_scalar::<i32, _>(&CreateTaskQueryView::new(
            project,
            owner,
            "Appel d'offres",
            "",
            TaskStatus::Todo,
            TaskPriority::Medium,
            None,
            None,
            &[],
        ))
        .await
        .unwrap();
    let comment: ActivityRow = db
        .fetch_one(&AddProjectCommentQueryView::new(project, owner, "  Vu  "))
        .await
        .unwrap();
    assert_eq!(comment.kind, "comment");
    assert_eq!(comment.message.as_deref(), Some("Vu"));
    assert_eq!(comment.author_id, Some(owner as i32));
    assert!(comment.author_name.is_some());
    db.fetch_one::<ActivityRow, _>(&AddProjectCommentQueryView::new(other, owner, "Ailleurs"))
        .await
        .unwrap();
    // The task is completed (hence archived) outside the API: its history stays in the feed.
    set_task_status(task as u64, "completed").await;

    let page: PagedRows<ActivityRow> = db
        .fetch_one(&GetProjectActivityQueryView::new(project, 100, 0))
        .await
        .unwrap();
    assert_eq!(page.total, 3, "{page:?}");
    let kinds: Vec<&str> = page.items.iter().map(|row| row.kind.as_str()).collect();
    assert_eq!(kinds, ["status_changed", "comment", "task_created"]);
    assert_eq!(page.items[0].task_id, Some(task));
    assert_eq!(page.items[0].task_title.as_deref(), Some("Appel d'offres"));
    assert_eq!(page.items[2].author_id, Some(owner as i32));
    assert!(page.items.iter().all(|row| row.created_at.ends_with('Z')));

    let second: PagedRows<ActivityRow> = db
        .fetch_one(&GetProjectActivityQueryView::new(project, 1, 1))
        .await
        .unwrap();
    assert_eq!((second.total, second.items.len()), (3, 1));
    assert_eq!(second.items[0].kind, "comment");
}
