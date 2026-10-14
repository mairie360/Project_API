use crate::common::fixtures::create_user;
use crate::common::get_smart_db;
use mairie360_api_lib::smart_db::SmartDatabase;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
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
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let owner = create_user(&db, "Owner", None).await;
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
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;

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
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let owner = create_user(&db, "Planner", None).await;
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
