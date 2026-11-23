//! MAIR-1087: the start date of a task, written at creation and by a PATCH, read by every task
//! read, and never after the due date (database `CHECK`, SQLSTATE `23514`).

use chrono::{DateTime, Utc};

use crate::common::get_smart_db;
use project_api::database::paged::PagedRows;
use project_api::database::project::create::view::CreateProjectQueryView;
use project_api::database::tasks::create_task::view::{
    CreateTaskQueryView, TaskPriority, TaskStatus,
};
use project_api::database::tasks::get_project_tasks::view::{GetProjectTasksQueryView, Task};
use project_api::database::tasks::get_task::view::GetTaskQueryView;
use project_api::database::tasks::get_visible_tasks::view::{
    GetVisibleTasksQueryView, TaskFilters, VisibleTaskRow,
};
use project_api::database::tasks::patch_task::view::{
    PatchTaskQueryView, TaskChanges, TASK_UPDATED,
};
use project_api::endpoints::db_error::{classify_db_error, DbFailure};

fn at(text: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(text)
        .unwrap()
        .with_timezone(&Utc)
}

#[tokio::test]
async fn mair_1087_the_start_date_is_stored_read_cleared_and_kept_before_the_due_date() {
    let db = get_smart_db().await;
    let project = db
        .fetch_scalar::<i32, _>(&CreateProjectQueryView::new("Voirie", None, 1))
        .await
        .unwrap() as u64;
    let create = |start: Option<DateTime<Utc>>, due: Option<DateTime<Utc>>| {
        CreateTaskQueryView::new(
            project,
            1,
            "Chantier",
            "",
            TaskStatus::Todo,
            TaskPriority::Medium,
            due,
            None,
            &[],
        )
        .with_start_date(start)
    };
    let task = db
        .fetch_scalar::<i32, _>(&create(
            Some(at("2026-10-01T08:00:00Z")),
            Some(at("2026-10-15T00:00:00Z")),
        ))
        .await
        .unwrap() as u64;

    let alone: Vec<Task> = db
        .fetch_all(&GetTaskQueryView::new(project, task))
        .await
        .unwrap();
    assert_eq!(alone[0].start_date(), Some(at("2026-10-01T08:00:00Z")));
    let page: PagedRows<Task> = db
        .fetch_one(&GetProjectTasksQueryView::new(project, 100, 0))
        .await
        .unwrap();
    assert_eq!(page.items[0].start_date(), Some(at("2026-10-01T08:00:00Z")));
    let visible: PagedRows<VisibleTaskRow> = db
        .fetch_one(&GetVisibleTasksQueryView::new(
            1,
            &TaskFilters::default(),
            500,
            0,
        ))
        .await
        .unwrap();
    let listed = visible
        .items
        .iter()
        .find(|row| row.task.id() as u64 == task)
        .unwrap();
    assert_eq!(listed.task.start_date(), Some(at("2026-10-01T08:00:00Z")));

    // A start date after the due date is refused at creation and by a PATCH of either date.
    let refused = db
        .fetch_scalar::<i32, _>(&create(
            Some(at("2026-10-16T00:00:00Z")),
            Some(at("2026-10-15T00:00:00Z")),
        ))
        .await
        .unwrap_err();
    assert_eq!(
        classify_db_error("test", &refused),
        DbFailure::CheckViolation
    );
    let patch = |changes: TaskChanges<'static>| PatchTaskQueryView::new(project, task, 1, changes);
    let late_start = db
        .fetch_scalar::<i32, _>(&patch(TaskChanges {
            start_date: Some(Some(at("2026-11-01T00:00:00Z"))),
            ..TaskChanges::default()
        }))
        .await
        .unwrap_err();
    assert_eq!(
        classify_db_error("test", &late_start),
        DbFailure::CheckViolation
    );
    let early_due = db
        .fetch_scalar::<i32, _>(&patch(TaskChanges {
            due_date: Some(at("2026-09-01T00:00:00Z")),
            ..TaskChanges::default()
        }))
        .await
        .unwrap_err();
    assert_eq!(
        classify_db_error("test", &early_due),
        DbFailure::CheckViolation
    );

    // Absent keeps it, `Some(None)` clears it.
    let renamed: i32 = db
        .fetch_scalar(&PatchTaskQueryView::new(
            project,
            task,
            1,
            TaskChanges {
                title: Some("Chantier 2"),
                ..TaskChanges::default()
            },
        ))
        .await
        .unwrap();
    assert_eq!(renamed, TASK_UPDATED);
    let kept: Vec<Task> = db
        .fetch_all(&GetTaskQueryView::new(project, task))
        .await
        .unwrap();
    assert_eq!(kept[0].start_date(), Some(at("2026-10-01T08:00:00Z")));
    let cleared: i32 = db
        .fetch_scalar(&patch(TaskChanges {
            start_date: Some(None),
            ..TaskChanges::default()
        }))
        .await
        .unwrap();
    assert_eq!(cleared, TASK_UPDATED);
    let read: Vec<Task> = db
        .fetch_all(&GetTaskQueryView::new(project, task))
        .await
        .unwrap();
    assert_eq!(read[0].start_date(), None);
    assert_eq!(read[0].due_date(), Some(at("2026-10-15T00:00:00Z")));
}
