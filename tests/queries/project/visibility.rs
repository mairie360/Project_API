use crate::common::fixtures::{create_group, create_user};
use crate::common::get_smart_db;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use project_api::database::project::create::view::CreateProjectQueryView;
use project_api::database::project::get_project::view::GetVisibleProjectQueryView;
use project_api::database::project::get_projects::view::{GetProjectsQueryView, ProjectView};
use project_api::database::users::add_user_to_project::view::AddUserToProjectQueryView;

#[tokio::test]
async fn test_project_visibility_follows_roles_membership_and_teams() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let owner = create_user(&db, "Owner", Some("User")).await;
    let member = create_user(&db, "Member", Some("User")).await;
    let outsider = create_user(&db, "Outsider", Some("User")).await;
    let teammate = create_user(&db, "Teammate", Some("User")).await;
    let responsable = create_user(&db, "Responsable", Some("Responsable")).await;
    let foreign_responsable = create_user(&db, "Foreign", Some("Responsable")).await;
    let admin = create_user(&db, "Admin", Some("Admin")).await;
    let maire = create_user(&db, "Maire", Some("Maire")).await;
    create_group(&db, owner, &[owner, teammate, responsable]).await;

    let project_id = db
        .fetch_scalar::<i32, _>(&CreateProjectQueryView::new("Visibilité", None, owner))
        .await
        .unwrap() as u64;
    db.execute(AddUserToProjectQueryView::new(project_id, member))
        .await
        .unwrap();

    for (user, expected) in [
        (owner, true),
        (member, true),
        (responsable, true),
        (admin, true),
        (maire, true),
        (outsider, false),
        (teammate, false),
        (foreign_responsable, false),
    ] {
        let listed: Vec<ProjectView> = db
            .fetch_all(&GetProjectsQueryView::new(user))
            .await
            .unwrap();
        let single: Vec<ProjectView> = db
            .fetch_all(&GetVisibleProjectQueryView::new(project_id, user))
            .await
            .unwrap();
        assert_eq!(
            listed
                .iter()
                .any(|project| project.id() as u64 == project_id),
            expected,
            "liste de l'utilisateur {user}"
        );
        assert_eq!(
            single.len() == 1,
            expected,
            "lecture par l'utilisateur {user}"
        );
    }
}

#[tokio::test]
async fn test_projects_are_listed_newest_first() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let owner = create_user(&db, "Owner", None).await;
    let first = db
        .fetch_scalar::<i32, _>(&CreateProjectQueryView::new("Premier", None, owner))
        .await
        .unwrap();
    let second = db
        .fetch_scalar::<i32, _>(&CreateProjectQueryView::new("Second", Some("desc"), owner))
        .await
        .unwrap();

    let projects: Vec<ProjectView> = db
        .fetch_all(&GetProjectsQueryView::new(owner))
        .await
        .unwrap();

    assert_eq!(
        projects.iter().map(ProjectView::id).collect::<Vec<_>>(),
        vec![second, first]
    );
    assert_eq!(
        projects[0],
        ProjectView::new(second, "Second", Some("desc"), "active")
    );
}

#[tokio::test]
async fn test_unknown_project_is_not_visible() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let admin = create_user(&db, "Admin", Some("Admin")).await;

    let projects: Vec<ProjectView> = db
        .fetch_all(&GetVisibleProjectQueryView::new(999_999, admin))
        .await
        .unwrap();

    assert!(projects.is_empty());
}
