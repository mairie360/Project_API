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
use project_api::database::tasks::fields::change_value::query::change_field_value_query;
use project_api::database::tasks::fields::change_value::view::ChangeFieldValueQueryView;
use project_api::database::tasks::fields::get_fields::query::get_task_custom_fields_query;
use project_api::database::tasks::fields::get_fields::view::{
    CustomeFieldsQueryResultView, GetTaskFieldsQueryView,
};
use project_api::database::tasks::get_project_tasks::view::{DynamicTaskField, FieldType};
use sqlx::types::Json;

#[sqlx::test]
async fn test_change_field_from_task_success() {
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

    let field = DynamicTaskField {
        label: "field1".to_string(),
        task_type: FieldType::Date,
        fields_options: Vec::new(),
    };

    let custom_fields = Json(std::collections::HashMap::from([(
        "field1".to_string(),
        field.clone(),
    )]));

    let view = AddFieldToTaskQueryView::new(task_id as u64, custom_fields);

    let result = add_field_to_task_query(view, pool.clone()).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );

    let changed_field = DynamicTaskField {
        label: "field2".to_string(),
        task_type: FieldType::Date,
        fields_options: Vec::new(),
    };

    let custom_fields = Json(std::collections::HashMap::from([(
        "field2".to_string(),
        changed_field.clone(),
    )]));

    let view = ChangeFieldValueQueryView::new(task_id as u64, custom_fields);

    let result = change_field_value_query(view, pool.clone()).await;

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

    let view = GetTaskFieldsQueryView::new(task_id as u64);
    let result = get_task_custom_fields_query(view, pool).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
    let fields_view: CustomeFieldsQueryResultView = result.unwrap();
    let fields = fields_view
        .custom_fields()
        .values()
        .cloned()
        .collect::<Vec<DynamicTaskField>>();
    assert_eq!(
        fields.len(),
        1,
        "Task must have 1 custome field, got: {:?}",
        fields
    );
    assert_ne!(
        fields[0], field,
        "Expected field to not match, got: {:?}",
        fields[0]
    );
    assert_eq!(
        fields[0], changed_field,
        "Expected changed_field to match, got: {:?}",
        fields[0]
    );
}

#[sqlx::test]
async fn test_change_fields_from_task_unknown_task() {
    let (_container, host) = get_shared_db().await;
    let pool = get_pool(host.to_string()).await;

    let changed_field = DynamicTaskField {
        label: "field1".to_string(),
        task_type: FieldType::Date,
        fields_options: Vec::new(),
    };

    let custom_fields = Json(std::collections::HashMap::from([(
        "field1".to_string(),
        changed_field.clone(),
    )]));

    let view = ChangeFieldValueQueryView::new(999, custom_fields);

    let result = change_field_value_query(view, pool.clone()).await;

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
