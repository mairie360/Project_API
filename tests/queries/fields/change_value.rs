use std::collections::HashMap;

use crate::common::get_smart_db;
use mairie360_api_lib::smart_db::SmartDatabase;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use project_api::database::project::create::view::CreateProjectQueryView;
use project_api::database::tasks::create_task::view::{
    CreateTaskQueryView, TaskPriority, TaskStatus,
};
use project_api::database::tasks::fields::add_field_to_task::view::AddFieldToTaskQueryView;
use project_api::database::tasks::fields::change_value::view::ChangeFieldValueQueryView;
use project_api::database::tasks::fields::get_fields::view::GetTaskFieldsQueryView;
use project_api::database::tasks::get_project_tasks::view::{DynamicTaskField, FieldType};

fn field(name: &str) -> DynamicTaskField {
    DynamicTaskField {
        label: name.to_string(),
        task_type: FieldType::Date,
        fields_options: Vec::new(),
    }
}

async fn create_task(db: &SmartDatabase) -> u64 {
    let view = CreateProjectQueryView::new("Test Project", Some("Test Description"), 1);
    let project_id = db.fetch_scalar::<i32, _>(&view).await.unwrap() as u64;

    let view = CreateTaskQueryView::new(
        project_id,
        "Test Task",
        TaskStatus::Todo,
        TaskPriority::Medium,
        Some(chrono::Utc::now()),
        Some(1),
    );
    db.fetch_scalar::<i32, _>(&view).await.unwrap() as u64
}

#[tokio::test]
async fn test_change_field_from_task_success() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let task_id = create_task(&db).await;

    let original = field("field1");
    let view = AddFieldToTaskQueryView::new(
        task_id,
        HashMap::from([("field1".to_string(), original.clone())]),
    );
    assert!(db.execute(view).await.is_ok());

    let changed = field("field2");
    let view = ChangeFieldValueQueryView::new(
        task_id,
        HashMap::from([("field2".to_string(), changed.clone())]),
    );
    let result = db.execute(view).await;
    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );

    let view = GetTaskFieldsQueryView::new(task_id);
    let result = db
        .fetch_one::<HashMap<String, DynamicTaskField>, _>(&view)
        .await;
    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );

    let fields: Vec<DynamicTaskField> = result.unwrap().into_values().collect();
    assert_eq!(
        fields.len(),
        1,
        "Task must have 1 custom field, got: {:?}",
        fields
    );
    assert_ne!(fields[0], original, "Expected field to not match original");
    assert_eq!(fields[0], changed, "Expected changed field to match");
}

#[tokio::test]
async fn test_change_fields_from_task_unknown_task() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;

    let view = ChangeFieldValueQueryView::new(
        999,
        HashMap::from([("field1".to_string(), field("field1"))]),
    );
    let result = db.execute(view).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
}
