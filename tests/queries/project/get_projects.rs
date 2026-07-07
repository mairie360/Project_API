use crate::common::get_pool; // Utilisation de ta fonction utilitaire existante
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use project_api::database::project::create::query::create_project_query;
use project_api::database::project::create::view::CreateProjectQueryView;
use project_api::database::project::get_projects::query::get_projects_query;
use project_api::database::project::get_projects::view::GetProjectsQueryView;

#[sqlx::test]
async fn test_get_project_success() {
    let (_container, host) = get_shared_db().await;
    let pool = get_pool(host.to_string()).await;

    let view = CreateProjectQueryView::new("Test Project", Some("Test Description"), 1);

    let result = create_project_query(view, pool.clone()).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );

    let view = GetProjectsQueryView::new(1);
    let result = get_projects_query(view, pool).await;

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

#[sqlx::test]
async fn test_get_project_none_description_success() {
    let (_container, host) = get_shared_db().await;
    let pool = get_pool(host.to_string()).await;

    let view = CreateProjectQueryView::new("Test Project", None, 1);

    let result = create_project_query(view, pool.clone()).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );

    let view = GetProjectsQueryView::new(1);
    let result = get_projects_query(view, pool).await;

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

#[sqlx::test]
async fn test_get_project_unknown_user() {
    let (_container, host) = get_shared_db().await;
    let pool = get_pool(host.to_string()).await;

    let view = GetProjectsQueryView::new(999);

    let result = get_projects_query(view, pool.clone()).await;

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
