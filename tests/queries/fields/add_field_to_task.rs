use std::collections::HashMap;

use crate::common::get_smart_db;
use mairie360_api_lib::smart_db::SmartDatabase;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use project_api::database::project::create::view::CreateProjectQueryView;
use project_api::database::tasks::create_task::view::{
    CreateTaskQueryView, TaskPriority, TaskStatus,
};
use project_api::database::tasks::fields::add_field_to_task::view::AddFieldToTaskQueryView;
use project_api::database::tasks::get_project_tasks::view::{DynamicTaskField, FieldType};

fn one_field(name: &str) -> HashMap<String, DynamicTaskField> {
    HashMap::from([(
        name.to_string(),
        DynamicTaskField {
            label: name.to_string(),
            task_type: FieldType::Date,
            fields_options: Vec::new(),
        },
    )])
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
async fn test_add_field_to_task_first_field_success() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let task_id = create_task(&db).await;

    let view = AddFieldToTaskQueryView::new(task_id, one_field("field1"));
    let result = db.execute(view).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_add_field_to_task_several_field_success() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let task_id = create_task(&db).await;

    let view = AddFieldToTaskQueryView::new(task_id, one_field("field1"));
    assert!(db.execute(view).await.is_ok());

    let view = AddFieldToTaskQueryView::new(task_id, one_field("field2"));
    let result = db.execute(view).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_add_field_to_task_unknown_task() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;

    let view = AddFieldToTaskQueryView::new(999, one_field("field1"));
    let result = db.execute(view).await;

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
}
