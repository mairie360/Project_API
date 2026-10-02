use crate::common::fixtures::create_user;
use crate::common::get_smart_db;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use project_api::database::paged::PagedRows;
use project_api::database::project::create::view::CreateProjectQueryView;
use project_api::database::project::get_projects::view::{GetProjectsQueryView, ProjectView};

#[tokio::test]
async fn test_get_project_success() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;

    let view = CreateProjectQueryView::new("Test Project", Some("Test Description"), 1);
    let result = db.fetch_scalar::<i32, _>(&view).await;
    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );

    let view = GetProjectsQueryView::new(1, 500, 0);
    let result = db
        .fetch_one::<PagedRows<ProjectView>, _>(&view)
        .await
        .map(|page| page.items);

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
    let projects = result.unwrap();
    assert!(
        !projects.is_empty(),
        "Expected projects to be non-empty, got: {:?}",
        projects
    );
}

#[tokio::test]
async fn test_get_project_none_description_success() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;

    let view = CreateProjectQueryView::new("Test Project", None, 1);
    let result = db.fetch_scalar::<i32, _>(&view).await;
    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );

    let view = GetProjectsQueryView::new(1, 500, 0);
    let result = db
        .fetch_one::<PagedRows<ProjectView>, _>(&view)
        .await
        .map(|page| page.items);

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
    let projects = result.unwrap();
    assert!(
        !projects.is_empty(),
        "Expected projects to be non-empty, got: {:?}",
        projects
    );
}

#[tokio::test]
async fn test_get_project_unknown_user() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;

    let view = GetProjectsQueryView::new(999_999, 100, 0);
    let result = db
        .fetch_one::<PagedRows<ProjectView>, _>(&view)
        .await
        .map(|page| page.items);

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
    let projects = result.unwrap();
    assert!(
        projects.is_empty(),
        "Expected projects to be empty, got: {:?}",
        projects
    );
}

#[tokio::test]
async fn test_get_projects_is_paginated_with_an_exact_total() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let owner = create_user(&db, "Pager", None).await;
    let mut ids = Vec::new();
    for title in ["A", "B", "C"] {
        ids.push(
            db.fetch_scalar::<i32, _>(&CreateProjectQueryView::new(title, None, owner))
                .await
                .unwrap(),
        );
    }

    let page: PagedRows<ProjectView> = db
        .fetch_one(&GetProjectsQueryView::new(owner, 2, 1))
        .await
        .unwrap();
    assert_eq!(page.total, 3);
    assert_eq!(
        page.items.iter().map(ProjectView::id).collect::<Vec<_>>(),
        vec![ids[1], ids[0]]
    );

    let past_the_end: PagedRows<ProjectView> = db
        .fetch_one(&GetProjectsQueryView::new(owner, 2, 10))
        .await
        .unwrap();
    assert_eq!(past_the_end.total, 3);
    assert!(past_the_end.items.is_empty());
}
