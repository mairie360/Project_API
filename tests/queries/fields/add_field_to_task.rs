use crate::common::get_pool; // Utilisation de ta fonction utilitaire existante
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use project_api::database::project::create::query::create_project_query;
use project_api::database::project::create::view::CreateProjectQueryView;
use project_api::database::tasks::create_task::query::create_task_query;
use project_api::database::tasks::create_task::view::{
    CreateTaskQueryView, TaskPriority, TaskStatus,
};
use project_api::database::tasks::fields::add_field_to_task::query::add_field_to_task_query;
use project_api::database::tasks::fields::add_field_to_task::view::AddFieldToTaskQueryView;
use project_api::database::tasks::get_project_tasks::view::{DynamicTaskField, FieldType};
use sqlx::types::Json;

#[sqlx::test]
async fn test_add_field_to_task_first_field_success() {
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

    let view = CreateTaskQueryView::new(
        project_id,
        "Test Task",
        TaskStatus::Todo,
        TaskPriority::Medium,
        Some(chrono::Utc::now()),
        Some(1),
    );
    let result = create_task_query(view, pool.clone()).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );

    let task_id = result.unwrap();

    let custom_fields = Json(std::collections::HashMap::from([(
        "field1".to_string(),
        DynamicTaskField {
            label: "field1".to_string(),
            task_type: FieldType::Date,
            fields_options: Vec::new(),
        },
    )]));

    let view = AddFieldToTaskQueryView::new(task_id as u64, custom_fields);

    let result = add_field_to_task_query(view, pool).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
    let rows_affected = result.unwrap();
    assert!(
        rows_affected > 0,
        "Expected rows_affected to be greater than 0, got: {}",
        rows_affected
    );
}

#[sqlx::test]
async fn test_add_field_to_task_several_field_success() {
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

    let view = CreateTaskQueryView::new(
        project_id,
        "Test Task",
        TaskStatus::Todo,
        TaskPriority::Medium,
        Some(chrono::Utc::now()),
        Some(1),
    );
    let result = create_task_query(view, pool.clone()).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );

    let task_id = result.unwrap();

    let custom_fields = Json(std::collections::HashMap::from([(
        "field1".to_string(),
        DynamicTaskField {
            label: "field1".to_string(),
            task_type: FieldType::Date,
            fields_options: Vec::new(),
        },
    )]));

    let view = AddFieldToTaskQueryView::new(task_id as u64, custom_fields);

    let result = add_field_to_task_query(view, pool.clone()).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
    let rows_affected = result.unwrap();
    assert!(
        rows_affected > 0,
        "Expected rows_affected to be greater than 0, got: {}",
        rows_affected
    );

    let custom_fields = Json(std::collections::HashMap::from([(
        "field1".to_string(),
        DynamicTaskField {
            label: "field1".to_string(),
            task_type: FieldType::Date,
            fields_options: Vec::new(),
        },
    )]));

    let view = AddFieldToTaskQueryView::new(task_id as u64, custom_fields);

    let result = add_field_to_task_query(view, pool).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
    let rows_affected = result.unwrap();
    assert!(
        rows_affected > 0,
        "Expected rows_affected to be greater than 0, got: {}",
        rows_affected
    );
}

#[sqlx::test]
async fn test_add_field_to_task_unknown_task() {
    let (_container, host) = get_shared_db().await;
    let pool = get_pool(host.to_string()).await;

    let custom_fields = Json(std::collections::HashMap::from([(
        "field1".to_string(),
        DynamicTaskField {
            label: "field1".to_string(),
            task_type: FieldType::Date,
            fields_options: Vec::new(),
        },
    )]));

    let view = AddFieldToTaskQueryView::new(999 as u64, custom_fields);

    let result = add_field_to_task_query(view, pool).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
    let rows_affected = result.unwrap();
    assert!(
        rows_affected == 0,
        "Expected rows_affected to be 0, got: {}",
        rows_affected
    );
}
