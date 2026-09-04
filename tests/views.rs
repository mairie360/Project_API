//! Tests unitaires des vues de requête (`src/database/**/view.rs`).
//!
//! Contrairement à `tests/queries/`, ces tests ne touchent ni Postgres ni Docker :
//! ils vérifient la construction des `QueryView` (accesseurs, ordre des
//! paramètres, SQL) et les conversions d'enums (`From<String>` / `Display`) ainsi
//! que la (dé)sérialisation des DTOs de résultat.

use std::collections::HashMap;

use mairie360_api_lib::database::db_interface::ApiRequestDto;

use project_api::database::project::create::view::CreateProjectQueryView;
use project_api::database::project::delete::view::DeleteProjectQueryView;
use project_api::database::project::get_projects::view::{GetProjectsQueryView, ProjectView};
use project_api::database::project::update_status::view::{
    ProjectStatus, UpdateProjectStatusQueryView,
};
use project_api::database::tasks::create_task::view::{
    CreateTaskQueryView, TaskPriority, TaskStatus,
};
use project_api::database::tasks::delete_task::view::DeleteTaskQueryView;
use project_api::database::tasks::fields::add_field_to_task::view::AddFieldToTaskQueryView;
use project_api::database::tasks::fields::change_value::view::ChangeFieldValueQueryView;
use project_api::database::tasks::fields::get_fields::view::GetTaskFieldsQueryView;
use project_api::database::tasks::get_project_tasks::view::{
    DynamicTaskField, FieldOption, FieldType, GetProjectTasksQueryView, Task,
};
use project_api::database::tasks::patch_task::view::PatchTaskQueryView;
use project_api::database::users::add_user_to_project::view::AddUserToProjectQueryView;
use project_api::database::users::get_project_users::view::GetProjectUsersQueryView;
use project_api::database::users::remove_user_from_project::view::RemoveUserFromProjectQueryView;

// ---------------------------------------------------------------------------
// project
// ---------------------------------------------------------------------------

#[test]
fn create_project_view_accessors() {
    let view = CreateProjectQueryView::new("Titre", Some("Desc"), 42);
    assert_eq!(view.title(), "Titre");
    assert_eq!(view.description(), "Desc");
    assert_eq!(view.owner_id(), 42);
    assert_eq!(view.query_params().len(), 3);
    assert!(view.query_sql().contains("INSERT INTO projects"));
}

#[test]
fn create_project_view_none_description_is_empty() {
    let view = CreateProjectQueryView::new("Titre", None, 1);
    assert_eq!(view.description(), "");
}

#[test]
fn delete_project_view_accessors() {
    let view = DeleteProjectQueryView::new(7);
    assert_eq!(view.project_id(), 7);
    assert_eq!(view.query_params().len(), 1);
    assert!(view.query_sql().contains("DELETE FROM projects"));
}

#[test]
fn get_projects_view_accessors() {
    let view = GetProjectsQueryView::new(9);
    assert_eq!(view.user_id(), 9);
    assert_eq!(view.query_params().len(), 1);
    assert!(view.query_sql().contains("project_members"));
}

#[test]
fn project_view_deserializes_and_exposes_fields() {
    let with_desc: ProjectView =
        serde_json::from_str(r#"{"id":3,"title":"P","description":"D","status":"active"}"#)
            .unwrap();
    assert_eq!(with_desc.id(), 3);
    assert_eq!(with_desc.title(), "P");
    assert_eq!(with_desc.description(), Some("D"));
    assert_eq!(with_desc.status(), "active");

    let no_desc: ProjectView =
        serde_json::from_str(r#"{"id":4,"title":"Q","description":null,"status":"completed"}"#)
            .unwrap();
    assert_eq!(no_desc.description(), None);
}

#[test]
fn update_status_view_accessors() {
    let view = UpdateProjectStatusQueryView::new(5, ProjectStatus::Suspended);
    assert_eq!(view.project_id(), 5);
    assert_eq!(view.status(), ProjectStatus::Suspended);
    assert_eq!(view.query_params().len(), 2);
    assert!(view.query_sql().contains("UPDATE projects SET status"));
}

#[test]
fn project_status_from_string_all_branches() {
    assert_eq!(
        ProjectStatus::from("suspended".to_string()),
        ProjectStatus::Suspended
    );
    assert_eq!(
        ProjectStatus::from("archived".to_string()),
        ProjectStatus::Archived
    );
    assert_eq!(
        ProjectStatus::from("completed".to_string()),
        ProjectStatus::Completed
    );
    assert_eq!(
        ProjectStatus::from("error".to_string()),
        ProjectStatus::Error
    );
    assert_eq!(
        ProjectStatus::from("active".to_string()),
        ProjectStatus::Active
    );
    assert_eq!(
        ProjectStatus::from("whatever".to_string()),
        ProjectStatus::Active
    );
}

#[test]
fn project_status_display_all_branches() {
    assert_eq!(ProjectStatus::Active.to_string(), "active");
    assert_eq!(ProjectStatus::Suspended.to_string(), "suspended");
    assert_eq!(ProjectStatus::Archived.to_string(), "archived");
    assert_eq!(ProjectStatus::Completed.to_string(), "completed");
    assert_eq!(ProjectStatus::Error.to_string(), "error");
}

#[test]
fn project_status_round_trips_through_view() {
    for status in [
        ProjectStatus::Active,
        ProjectStatus::Suspended,
        ProjectStatus::Archived,
        ProjectStatus::Completed,
        ProjectStatus::Error,
    ] {
        let view = UpdateProjectStatusQueryView::new(1, status);
        assert_eq!(view.status(), status);
    }
}

// ---------------------------------------------------------------------------
// tasks
// ---------------------------------------------------------------------------

#[test]
fn create_task_view_accessors() {
    let view = CreateTaskQueryView::new(
        3,
        "Ma tache",
        TaskStatus::InProgress,
        TaskPriority::High,
        Some(chrono::Utc::now()),
        Some(11),
    );
    assert_eq!(view.project_id(), 3);
    assert_eq!(view.title(), "Ma tache");
    assert_eq!(view.query_params().len(), 6);
    assert!(view.query_sql().contains("INSERT INTO tasks"));
}

#[test]
fn create_task_view_without_optionals() {
    let view = CreateTaskQueryView::new(1, "T", TaskStatus::Todo, TaskPriority::Low, None, None);
    assert_eq!(view.title(), "T");
    assert_eq!(view.query_params().len(), 6);
}

#[test]
fn task_status_from_string_all_branches() {
    assert_eq!(TaskStatus::from("todo".to_string()), TaskStatus::Todo);
    assert_eq!(
        TaskStatus::from("in_progress".to_string()),
        TaskStatus::InProgress
    );
    assert_eq!(
        TaskStatus::from("completed".to_string()),
        TaskStatus::Completed
    );
    assert_eq!(TaskStatus::from("nope".to_string()), TaskStatus::Error);
}

#[test]
fn task_status_display_all_branches() {
    assert_eq!(TaskStatus::Todo.to_string(), "todo");
    assert_eq!(TaskStatus::InProgress.to_string(), "in_progress");
    assert_eq!(TaskStatus::Completed.to_string(), "completed");
    assert_eq!(TaskStatus::Error.to_string(), "error");
}

#[test]
fn task_priority_from_string_all_branches() {
    assert_eq!(TaskPriority::from("low".to_string()), TaskPriority::Low);
    assert_eq!(
        TaskPriority::from("medium".to_string()),
        TaskPriority::Medium
    );
    assert_eq!(TaskPriority::from("high".to_string()), TaskPriority::High);
    assert_eq!(TaskPriority::from("???".to_string()), TaskPriority::Error);
}

#[test]
fn task_priority_display_all_branches() {
    assert_eq!(TaskPriority::Low.to_string(), "low");
    assert_eq!(TaskPriority::Medium.to_string(), "medium");
    assert_eq!(TaskPriority::High.to_string(), "high");
    assert_eq!(TaskPriority::Error.to_string(), "error");
}

#[test]
fn delete_task_view_accessors() {
    let view = DeleteTaskQueryView::new(8);
    assert_eq!(view.task_id(), 8);
    assert_eq!(view.query_params().len(), 1);
    assert!(view.query_sql().contains("DELETE FROM tasks"));
}

#[test]
fn get_project_tasks_view_accessors() {
    let view = GetProjectTasksQueryView::new(12);
    assert_eq!(view.project_id(), 12);
    assert_eq!(view.query_params().len(), 1);
    assert!(view.query_sql().contains("FROM tasks WHERE project_id"));
}

#[test]
fn patch_task_view_accessors() {
    let due = chrono::NaiveDate::from_ymd_opt(2024, 1, 2)
        .unwrap()
        .and_hms_opt(3, 4, 5)
        .unwrap();
    let view = PatchTaskQueryView::new(
        99,
        Some("nouveau titre"),
        Some(TaskStatus::Completed),
        Some(TaskPriority::Medium),
        Some(due),
        Some(7),
        None,
    );
    assert_eq!(view.task_id(), 99);
    assert_eq!(view.query_params().len(), 6);
    assert!(view.query_sql().contains("UPDATE tasks SET"));
}

#[test]
fn patch_task_view_all_none() {
    let view = PatchTaskQueryView::new(1, None, None, None, None, None, None);
    assert_eq!(view.task_id(), 1);
    assert_eq!(view.query_params().len(), 6);
}

#[test]
fn task_dto_deserializes_with_all_fields() {
    let json = r#"{
        "id": 1,
        "title": "T",
        "status": "todo",
        "priority": "high",
        "created_at": "2024-01-01T00:00:00",
        "assigned_to": 5,
        "custom_fields": {
            "col": {"label": "L", "task_type": "date", "fields_options": []}
        }
    }"#;
    let task: Task = serde_json::from_str(json).unwrap();
    assert_eq!(task.id(), 1);
    assert_eq!(task.title(), "T");
    assert_eq!(task.status(), "todo");
    assert_eq!(task.priority(), "high");
    assert_eq!(task.created_at(), Some("2024-01-01T00:00:00"));
    assert_eq!(task.assigned_to(), Some(5));
    assert_eq!(task.custom_fields().len(), 1);
    assert_eq!(task.custom_fields()["col"].task_type, FieldType::Date);
}

#[test]
fn task_dto_deserializes_with_missing_optionals() {
    let task: Task =
        serde_json::from_str(r#"{"id":2,"title":"T","status":"todo","priority":"low"}"#).unwrap();
    assert_eq!(task.created_at(), None);
    assert_eq!(task.assigned_to(), None);
    assert!(task.custom_fields().is_empty());
}

#[test]
fn field_type_deserialization() {
    assert_eq!(
        serde_json::from_str::<FieldType>(r#""date""#).unwrap(),
        FieldType::Date
    );
    assert_eq!(
        serde_json::from_str::<FieldType>(r#""checkbox""#).unwrap(),
        FieldType::Checkbox
    );
    assert_eq!(
        serde_json::from_str::<FieldType>(r#""select""#).unwrap(),
        FieldType::Select
    );
    // tout type inconnu retombe sur `Unknown` (`#[serde(other)]`)
    assert_eq!(
        serde_json::from_str::<FieldType>(r#""radio""#).unwrap(),
        FieldType::Unknown
    );
}

// ---------------------------------------------------------------------------
// tasks / fields
// ---------------------------------------------------------------------------

fn sample_fields() -> HashMap<String, DynamicTaskField> {
    let mut map = HashMap::new();
    map.insert(
        "priorite".to_string(),
        DynamicTaskField {
            label: "Priorité".to_string(),
            task_type: FieldType::Select,
            fields_options: vec![FieldOption {
                option: serde_json::json!("urgent"),
                is_selected: true,
            }],
        },
    );
    map
}

#[test]
fn add_field_to_task_view_accessors() {
    let view = AddFieldToTaskQueryView::new(4, sample_fields());
    assert_eq!(view.task_id(), 4);
    assert_eq!(view.query_params().len(), 2);
    assert!(view.query_sql().contains("custom_fields"));
}

#[test]
fn change_field_value_view_accessors() {
    let view = ChangeFieldValueQueryView::new(6, sample_fields());
    assert_eq!(view.task_id(), 6);
    assert_eq!(view.query_params().len(), 2);
    assert!(view.query_sql().contains("SET custom_fields = $2::jsonb"));
}

#[test]
fn get_task_fields_view_accessors() {
    let view = GetTaskFieldsQueryView::new(10);
    assert_eq!(view.id(), 10);
    assert_eq!(view.query_params().len(), 1);
    assert!(view.query_sql().contains("custom_fields"));
}

// ---------------------------------------------------------------------------
// users (appartenance à un projet)
// ---------------------------------------------------------------------------

#[test]
fn add_user_to_project_view_accessors() {
    let view = AddUserToProjectQueryView::new(2, 3);
    assert_eq!(view.project_id(), 2);
    assert_eq!(view.user_id(), 3);
    assert_eq!(view.query_params().len(), 2);
    assert!(view.query_sql().contains("INSERT INTO project_members"));
}

#[test]
fn get_project_users_view_accessors() {
    let view = GetProjectUsersQueryView::new(15);
    assert_eq!(view.project_id(), 15);
    assert_eq!(view.query_params().len(), 1);
    assert!(view
        .query_sql()
        .contains("FROM project_members WHERE project_id"));
}

#[test]
fn remove_user_from_project_view_accessors() {
    let view = RemoveUserFromProjectQueryView::new(2, 9);
    assert_eq!(view.project_id(), 2);
    assert_eq!(view.user_id(), 9);
    assert_eq!(view.query_params().len(), 2);
    assert!(view.query_sql().contains("DELETE FROM project_members"));
}
