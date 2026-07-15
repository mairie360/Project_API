use crate::common::get_pool; // Utilisation de ta fonction utilitaire existante
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use project_api::database::project::create::query::create_project_query;
use project_api::database::project::create::view::CreateProjectQueryView;
use project_api::database::project::delete::query::delete_project_query;
use project_api::database::project::delete::view::DeleteProjectQueryView;

#[sqlx::test]
async fn test_delete_project_success() {
    let (_container, host) = get_shared_db().await;
    let pool = get_pool(host.to_string()).await;

    let view = CreateProjectQueryView::new("Test Project", Some("Test Description"), 1);

    let result = create_project_query(view, pool.clone()).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
    let project_id = result.unwrap() as u64;

    let view = DeleteProjectQueryView::new(project_id);
    let result = delete_project_query(view, pool).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
    let rows_affected = result.unwrap();
    assert_eq!(
        rows_affected, 1,
        "Expected rows_affected to be 1, got: {}",
        rows_affected
    );
}

#[sqlx::test]
async fn test_delete_project_unknown_project() {
    let (_container, host) = get_shared_db().await;
    let pool = get_pool(host.to_string()).await;

    let view = DeleteProjectQueryView::new(999);
    let result = delete_project_query(view, pool).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
    let rows_affected = result.unwrap();
    assert_eq!(
        rows_affected, 0,
        "Expected rows_affected to be 0, got: {}",
        rows_affected
    );
}
