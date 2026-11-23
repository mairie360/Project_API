//! MAIR-1086: the milestones of a project, through the query views and the `project_api` role.

use chrono::NaiveDate;

use crate::common::fixtures::create_user;
use crate::common::get_smart_db;
use project_api::database::milestones::view::{
    CreateMilestoneQueryView, DeleteMilestoneQueryView, GetProjectMilestonesQueryView,
    MilestoneChanges, MilestoneRow, PatchMilestoneQueryView,
};
use project_api::database::paged::PagedRows;
use project_api::database::project::create::view::CreateProjectQueryView;

fn day(text: &str) -> NaiveDate {
    NaiveDate::parse_from_str(text, "%Y-%m-%d").unwrap()
}

#[tokio::test]
async fn mair_1086_milestones_are_listed_by_due_date_updated_and_soft_deleted() {
    let db = get_smart_db().await;
    let owner = create_user("Planner", None).await;
    let editor = create_user("Editor", None).await;
    let project = db
        .fetch_scalar::<i32, _>(&CreateProjectQueryView::new("Médiathèque", None, owner))
        .await
        .unwrap() as u64;
    let other = db
        .fetch_scalar::<i32, _>(&CreateProjectQueryView::new("Voirie", None, owner))
        .await
        .unwrap() as u64;

    let late: MilestoneRow = db
        .fetch_one(&CreateMilestoneQueryView::new(
            project,
            owner,
            "Livraison",
            day("2027-06-30"),
            false,
        ))
        .await
        .unwrap();
    assert_eq!(late.project_id as u64, project);
    assert_eq!((late.title.as_str(), late.reached), ("Livraison", false));
    assert_eq!(
        (late.created_by, late.updated_by),
        (Some(owner as i32), Some(owner as i32))
    );
    let early: MilestoneRow = db
        .fetch_one(&CreateMilestoneQueryView::new(
            project,
            owner,
            "Appel d'offres",
            day("2026-12-15"),
            true,
        ))
        .await
        .unwrap();
    db.fetch_one::<MilestoneRow, _>(&CreateMilestoneQueryView::new(
        other,
        owner,
        "Ailleurs",
        day("2026-01-01"),
        false,
    ))
    .await
    .unwrap();

    let page: PagedRows<MilestoneRow> = db
        .fetch_one(&GetProjectMilestonesQueryView::new(project, 100, 0))
        .await
        .unwrap();
    assert_eq!(page.total, 2);
    let ids: Vec<i32> = page.items.iter().map(|m| m.id).collect();
    assert_eq!(ids, vec![early.id, late.id], "by due date");
    assert_eq!(page.items[0].due_date, day("2026-12-15"));
    let second: PagedRows<MilestoneRow> = db
        .fetch_one(&GetProjectMilestonesQueryView::new(project, 1, 1))
        .await
        .unwrap();
    assert_eq!((second.total, second.items.len()), (2, 1));
    assert_eq!(second.items[0].id, late.id);

    // A partial update keeps what it does not carry and is signed by its author.
    let updated: Vec<MilestoneRow> = db
        .fetch_all(&PatchMilestoneQueryView::new(
            project,
            late.id as u64,
            editor,
            &MilestoneChanges {
                reached: Some(true),
                ..MilestoneChanges::default()
            },
        ))
        .await
        .unwrap();
    assert_eq!(updated.len(), 1);
    assert!(updated[0].reached);
    assert_eq!(updated[0].title, "Livraison");
    assert_eq!(updated[0].due_date, day("2027-06-30"));
    assert_eq!(updated[0].updated_by, Some(editor as i32));

    // Another project's id never reaches the milestone.
    let crossed: Vec<MilestoneRow> = db
        .fetch_all(&PatchMilestoneQueryView::new(
            other,
            late.id as u64,
            editor,
            &MilestoneChanges {
                title: Some("Volé"),
                ..MilestoneChanges::default()
            },
        ))
        .await
        .unwrap();
    assert!(crossed.is_empty());
    assert!(!db
        .fetch_scalar::<bool, _>(&DeleteMilestoneQueryView::new(
            other,
            late.id as u64,
            editor
        ))
        .await
        .unwrap());

    // Deleted once, then neither listed, updated nor deleted again.
    assert!(db
        .fetch_scalar::<bool, _>(&DeleteMilestoneQueryView::new(
            project,
            late.id as u64,
            editor
        ))
        .await
        .unwrap());
    assert!(!db
        .fetch_scalar::<bool, _>(&DeleteMilestoneQueryView::new(
            project,
            late.id as u64,
            editor
        ))
        .await
        .unwrap());
    let page: PagedRows<MilestoneRow> = db
        .fetch_one(&GetProjectMilestonesQueryView::new(project, 100, 0))
        .await
        .unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.items[0].id, early.id);
    let gone: Vec<MilestoneRow> = db
        .fetch_all(&PatchMilestoneQueryView::new(
            project,
            late.id as u64,
            editor,
            &MilestoneChanges {
                reached: Some(false),
                ..MilestoneChanges::default()
            },
        ))
        .await
        .unwrap();
    assert!(gone.is_empty());
}
