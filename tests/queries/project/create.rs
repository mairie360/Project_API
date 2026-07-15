use crate::common::get_pool; // Utilisation de ta fonction utilitaire existante
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use project_api::database::project::create::query::create_project_query;
use project_api::database::project::create::view::CreateProjectQueryView;

#[sqlx::test]
async fn test_create_project_success() {
    let (_container, host) = get_shared_db().await;
    let pool = get_pool(host.to_string()).await;

    let view = CreateProjectQueryView::new("Test Project", Some("Test Description"), 1);

    let result = create_project_query(view, pool).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
    let project_id = result.unwrap();
    assert!(
        project_id != 0,
        "Expected project_id to be non-zero, got: {}",
        project_id
    );
}

#[sqlx::test]
async fn test_create_project_none_description_success() {
    let (_container, host) = get_shared_db().await;
    let pool = get_pool(host.to_string()).await;

    let view = CreateProjectQueryView::new("Test Project", None, 1);

    let result = create_project_query(view, pool).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
    let project_id = result.unwrap();
    assert!(
        project_id != 0,
        "Expected project_id to be non-zero, got: {}",
        project_id
    );
}

#[sqlx::test]
async fn test_create_project_unknown_owner() {
    let (_container, host) = get_shared_db().await;
    let pool = get_pool(host.to_string()).await;

    let view = CreateProjectQueryView::new("Test Project", Some("Test Description"), 999);

    let result = create_project_query(view, pool).await;

    assert!(
        result.is_err(),
        "Expected result to be Err, got: {:?}",
        result
    );
}
