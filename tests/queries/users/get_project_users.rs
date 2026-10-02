use crate::common::fixtures::create_user;
use crate::common::get_smart_db;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use project_api::database::paged::PagedRows;
use project_api::database::project::create::view::CreateProjectQueryView;
use project_api::database::users::add_user_to_project::view::AddUserToProjectQueryView;
use project_api::database::users::get_project_users::view::{
    GetProjectUsersQueryView, ProjectMemberRow,
};

#[tokio::test]
async fn test_get_user_from_project_success() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;

    let view = CreateProjectQueryView::new("Test Project", Some("Test Description"), 1);
    let project_id = db.fetch_scalar::<i32, _>(&view).await.unwrap() as u64;

    let view = AddUserToProjectQueryView::new(project_id, 2);
    assert!(db.execute(view).await.is_ok());

    let view = GetProjectUsersQueryView::new(project_id, 100, 0);
    let result = db
        .fetch_one::<PagedRows<ProjectMemberRow>, _>(&view)
        .await
        .map(|page| page.items);

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
    let users = result.unwrap();
    assert_eq!(users.len(), 1, "Expected one member, got: {:?}", users);
    assert_eq!(users[0].id, 2);
    assert!(users[0]
        .name
        .as_deref()
        .is_some_and(|name| !name.is_empty()));
}

#[tokio::test]
async fn test_get_user_from_no_user_project_success() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;

    let view = CreateProjectQueryView::new("Test Project", Some("Test Description"), 1);
    let project_id = db.fetch_scalar::<i32, _>(&view).await.unwrap() as u64;

    let view = GetProjectUsersQueryView::new(project_id, 100, 0);
    let result = db
        .fetch_one::<PagedRows<ProjectMemberRow>, _>(&view)
        .await
        .map(|page| page.items);

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
    let users = result.unwrap();
    assert!(
        users.is_empty(),
        "Expected users to be empty, got: {:?}",
        users
    );
}

#[tokio::test]
async fn test_get_users_from_project_unknown_project() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;

    let view = GetProjectUsersQueryView::new(999, 100, 0);
    let result = db
        .fetch_one::<PagedRows<ProjectMemberRow>, _>(&view)
        .await
        .map(|page| page.items);

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
    let users = result.unwrap();
    assert!(
        users.is_empty(),
        "Expected users to be empty, got: {:?}",
        users
    );
}

#[tokio::test]
async fn test_get_users_from_project_is_paged_with_exact_total() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let owner = create_user(&db, "Owner", Some("Responsable")).await;
    let project_id = db
        .fetch_scalar::<i32, _>(&CreateProjectQueryView::new("Membres", None, owner))
        .await
        .unwrap() as u64;
    for name in ["Alice", "Bruno", "Chloé"] {
        let member = create_user(&db, name, None).await;
        db.execute(AddUserToProjectQueryView::new(project_id, member))
            .await
            .unwrap();
    }

    let first: PagedRows<ProjectMemberRow> = db
        .fetch_one(&GetProjectUsersQueryView::new(project_id, 2, 0))
        .await
        .unwrap();
    assert_eq!(first.total, 3);
    assert_eq!(first.items.len(), 2);
    let second: PagedRows<ProjectMemberRow> = db
        .fetch_one(&GetProjectUsersQueryView::new(project_id, 2, 2))
        .await
        .unwrap();
    assert_eq!(second.total, 3);
    assert_eq!(second.items.len(), 1);
    assert!(!first.items.contains(&second.items[0]));
    let past_the_end: PagedRows<ProjectMemberRow> = db
        .fetch_one(&GetProjectUsersQueryView::new(project_id, 2, 10))
        .await
        .unwrap();
    assert_eq!(past_the_end.total, 3);
    assert!(past_the_end.items.is_empty());
}
