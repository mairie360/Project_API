use crate::common::fixtures::create_user;
use crate::common::get_smart_db;
use mairie360_api_lib::smart_db::SmartDatabase;
use project_api::database::project::create::view::CreateProjectQueryView;
use project_api::database::project::get_project::view::GetVisibleProjectQueryView;
use project_api::database::project::get_projects::view::ProjectView;
use project_api::database::project::update::view::UpdateProjectQueryView;
use project_api::database::project::update_status::view::ProjectStatus;
use project_api::database::tasks::create_task::view::TaskPriority;

async fn read(db: &SmartDatabase, project_id: u64, user_id: u64) -> ProjectView {
    db.fetch_one::<ProjectView, _>(&GetVisibleProjectQueryView::new(project_id, user_id))
        .await
        .unwrap()
}

#[tokio::test]
async fn test_update_project_keeps_missing_fields_and_can_clear_description() {
    let db = get_smart_db().await;
    let owner = create_user("Owner", None).await;
    let project_id = db
        .fetch_scalar::<i32, _>(&CreateProjectQueryView::new(
            "Voirie",
            Some("Rue des écoles"),
            owner,
        ))
        .await
        .unwrap();
    let renamed: bool = db
        .fetch_scalar(&UpdateProjectQueryView::new(
            project_id as u64,
            Some("Voirie 2027"),
            None,
            Some(ProjectStatus::Suspended),
        ))
        .await
        .unwrap();
    assert!(renamed);
    assert_eq!(
        read(&db, project_id as u64, owner).await,
        ProjectView::new(
            project_id,
            "Voirie 2027",
            Some("Rue des écoles"),
            "suspended"
        )
    );

    let cleared: bool = db
        .fetch_scalar(&UpdateProjectQueryView::new(
            project_id as u64,
            None,
            Some(""),
            None,
        ))
        .await
        .unwrap();
    assert!(cleared);
    assert_eq!(
        read(&db, project_id as u64, owner).await,
        ProjectView::new(project_id, "Voirie 2027", None, "suspended")
    );
}

#[tokio::test]
async fn test_update_unknown_project() {
    let db = get_smart_db().await;

    let updated: bool = db
        .fetch_scalar(&UpdateProjectQueryView::new(999_999, Some("X"), None, None))
        .await
        .unwrap();

    assert!(!updated);
}

/// MAIR-558: a project created `todo` / `high` is read back as such, and an update of its priority is
/// kept by the next update that does not carry one.
#[tokio::test]
async fn mair_558_the_status_and_the_priority_given_are_stored() {
    let db = get_smart_db().await;
    let owner = create_user("Planner", None).await;
    let project_id = db
        .fetch_scalar::<i32, _>(
            &CreateProjectQueryView::new("Kermesse", None, owner)
                .with_status(ProjectStatus::Todo)
                .with_priority(TaskPriority::High),
        )
        .await
        .unwrap();
    let project = read(&db, project_id as u64, owner).await;
    assert_eq!((project.status(), project.priority()), ("todo", "high"));

    let updated: bool = db
        .fetch_scalar(
            &UpdateProjectQueryView::new(
                project_id as u64,
                None,
                None,
                Some(ProjectStatus::Review),
            )
            .with_priority(TaskPriority::Low),
        )
        .await
        .unwrap();
    assert!(updated);
    let project = read(&db, project_id as u64, owner).await;
    assert_eq!((project.status(), project.priority()), ("review", "low"));

    let renamed: bool = db
        .fetch_scalar(&UpdateProjectQueryView::new(
            project_id as u64,
            Some("Kermesse 2027"),
            None,
            None,
        ))
        .await
        .unwrap();
    assert!(renamed);
    let project = read(&db, project_id as u64, owner).await;
    assert_eq!((project.status(), project.priority()), ("review", "low"));
}

/// MAIR-1085: the budget is stored in cents, kept by an update that does not carry it, cleared by
/// `Some(None)`, and read back by the project detail and the list.
#[tokio::test]
async fn mair_1085_the_budget_is_set_kept_and_cleared() {
    use project_api::database::project::get_projects::view::{GetProjectsQueryView, ProjectsPage};

    let db = get_smart_db().await;
    let owner = create_user("Treasurer", None).await;
    let project_id = db
        .fetch_scalar::<i32, _>(&CreateProjectQueryView::new("Médiathèque", None, owner))
        .await
        .unwrap() as u64;
    let project = read(&db, project_id, owner).await;
    assert_eq!(
        (project.budget_voted(), project.budget_committed()),
        (None, None)
    );

    let budget = |voted, committed| {
        UpdateProjectQueryView::new(project_id, None, None, None).with_budget(voted, committed)
    };
    assert!(db
        .fetch_scalar::<bool, _>(&budget(Some(Some(1_250_000)), Some(Some(480_000))))
        .await
        .unwrap());
    let project = read(&db, project_id, owner).await;
    assert_eq!(
        (project.budget_voted(), project.budget_committed()),
        (Some(1_250_000), Some(480_000))
    );
    let page = db
        .fetch_one::<ProjectsPage, _>(&GetProjectsQueryView::new(owner, 100, 0))
        .await
        .unwrap();
    let listed = page
        .items
        .iter()
        .find(|item| item.id() as u64 == project_id)
        .unwrap();
    assert_eq!(
        (listed.budget_voted(), listed.budget_committed()),
        (Some(1_250_000), Some(480_000))
    );

    // Kept by an update without budget, then the committed amount alone is cleared.
    assert!(db
        .fetch_scalar::<bool, _>(&UpdateProjectQueryView::new(
            project_id,
            Some("Médiathèque 2027"),
            None,
            None
        ))
        .await
        .unwrap());
    assert!(db
        .fetch_scalar::<bool, _>(&budget(None, Some(None)))
        .await
        .unwrap());
    let project = read(&db, project_id, owner).await;
    assert_eq!(
        (project.budget_voted(), project.budget_committed()),
        (Some(1_250_000), None)
    );

    // The database refuses a negative amount (CHECK, 23514) even if a caller skipped the validation.
    assert!(db
        .fetch_scalar::<bool, _>(&budget(Some(Some(-1)), None))
        .await
        .is_err());
}
