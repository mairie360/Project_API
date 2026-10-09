//! Tests unitaires des vues de requête (`src/database/**/view.rs`).
//!
//! Contrairement à `tests/queries/`, ces tests ne touchent ni Postgres ni Docker :
//! ils vérifient la construction des `QueryView` (accesseurs, ordre des
//! paramètres, SQL) et les conversions d'enums (`From<String>` / `Display`) ainsi
//! que la (dé)sérialisation des DTOs de résultat.

use mairie360_api_lib::database::db_interface::ApiRequestDto;

use project_api::database::project::create::view::CreateProjectQueryView;
use project_api::database::project::delete::view::DeleteProjectQueryView;
use project_api::database::project::get_projects::view::{GetProjectsQueryView, ProjectView};
use project_api::database::project::update_status::view::{
    ProjectStatus, UpdateProjectStatusQueryView,
};
use project_api::database::tasks::collaboration::view::{TaskHistoryEntry, TaskHistoryRow};
use project_api::database::tasks::create_task::view::{
    CreateTaskQueryView, TaskPriority, TaskStatus,
};
use project_api::database::tasks::delete_task::view::DeleteTaskQueryView;
use project_api::database::tasks::get_project_tasks::view::{
    DynamicTaskField, FieldType, GetProjectTasksQueryView, Task,
};
use project_api::database::tasks::patch_task::view::{PatchTaskQueryView, TaskChanges};
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
    let view = GetProjectsQueryView::new(9, 50, 100);
    assert_eq!(view.user_id(), 9);
    assert_eq!(view.query_params().len(), 8);
    assert_eq!(view.query_params()[6].as_i64(), 50);
    assert_eq!(view.query_params()[7].as_i64(), 100);
    assert!(view.query_sql().contains("project_members"));
    assert!(view
        .query_sql()
        .contains("'total', (SELECT count(*) FROM filtered)"));
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
        5,
        "Ma tache",
        "Ma description",
        TaskStatus::InProgress,
        TaskPriority::High,
        Some(chrono::Utc::now()),
        Some(11),
        &[],
    );
    assert_eq!(view.project_id(), 3);
    assert_eq!(view.title(), "Ma tache");
    assert_eq!(view.query_params().len(), 9);
    assert_eq!(view.query_params()[7].as_text(), "Ma description");
    assert_eq!(view.query_params()[8].as_i32(), 5);
    assert!(view.query_sql().contains("INSERT INTO tasks"));
    assert!(view.query_sql().contains("project_members"));
}

#[test]
fn create_task_view_without_optionals() {
    let view = CreateTaskQueryView::new(
        1,
        2,
        "T",
        "",
        TaskStatus::Todo,
        TaskPriority::Low,
        None,
        None,
        &[],
    );
    assert_eq!(view.title(), "T");
    assert_eq!(view.query_params().len(), 9);
    assert_eq!(view.query_params()[6].as_text(), r#"{"fields":[]}"#);
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
    let view = GetProjectTasksQueryView::new(12, 100, 0);
    assert_eq!(view.project_id(), 12);
    assert_eq!(view.query_params().len(), 3);
    assert!(view.query_sql().contains("FROM tasks WHERE project_id"));
    assert!(view.query_sql().contains("description"));
}

#[test]
fn patch_task_view_accessors() {
    let due = chrono::DateTime::parse_from_rfc3339("2024-01-02T03:04:05Z")
        .unwrap()
        .with_timezone(&chrono::Utc);
    let fields = [DynamicTaskField {
        label: "Budget".to_string(),
        task_type: FieldType::Select,
        fields_options: vec![],
    }];
    let view = PatchTaskQueryView::new(
        4,
        99,
        3,
        TaskChanges {
            title: Some("nouveau titre"),
            description: Some(""),
            status: Some(TaskStatus::Completed),
            priority: Some(TaskPriority::Medium),
            due_date: Some(due),
            assigned_to: Some(Some(7)),
            fields: Some(&fields),
        },
    );
    assert_eq!(view.task_id(), 99);
    assert_eq!(view.project_id(), 4);
    assert_eq!(view.query_params().len(), 12);
    assert!(view.query_params()[6].as_bool());
    assert_eq!(view.query_params()[7].as_option_i32(), Some(7));
    // An empty description is a change (it clears the description), not an absence.
    assert!(view.query_params()[8].as_bool());
    assert_eq!(view.query_params()[9].as_text(), "");
    assert!(view.query_params()[10].as_text().contains("Budget"));
    assert_eq!(view.query_params()[11].as_i32(), 3);
    assert!(view.query_sql().contains("UPDATE tasks SET"));
    assert!(view.query_sql().contains("updated_by = $12"));
}

#[test]
fn patch_task_view_distinguishes_absent_and_cleared_assignee() {
    let absent = PatchTaskQueryView::new(1, 2, 3, TaskChanges::default());
    let cleared = PatchTaskQueryView::new(
        1,
        2,
        3,
        TaskChanges {
            assigned_to: Some(None),
            ..TaskChanges::default()
        },
    );
    assert!(!absent.query_params()[6].as_bool());
    assert!(!absent.query_params()[8].as_bool());
    assert_eq!(absent.query_params()[10].as_text(), "");
    assert!(cleared.query_params()[6].as_bool());
    assert_eq!(cleared.query_params()[7].as_option_i32(), None);
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
        "due_date": "2024-02-03T04:05:06+00:00",
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
    assert_eq!(
        task.due_date().map(|d| d.to_rfc3339()),
        Some("2024-02-03T04:05:06+00:00".to_string())
    );
    assert_eq!(task.fields().len(), 1);
    assert_eq!(task.fields()[0].task_type, FieldType::Date);
}

#[test]
fn task_fields_read_the_ordered_list_and_ignore_collaboration_entries() {
    let task: Task = serde_json::from_str(
        r#"{"id":3,"title":"T","status":"todo","priority":"low","custom_fields":{
            "fields":[
                {"label":"Date","task_type":"date","fields_options":[]},
                {"label":"Urgent","task_type":"select","fields_options":[]},
                {"label":"bad"}
            ],
            "comments":[{"id":"comment-1"}],
            "history":[]
        }}"#,
    )
    .unwrap();
    assert_eq!(
        task.fields()
            .iter()
            .map(|field| field.label.as_str())
            .collect::<Vec<_>>(),
        vec!["Date", "Urgent"]
    );
}

#[test]
fn task_dto_deserializes_with_missing_optionals() {
    let task: Task =
        serde_json::from_str(r#"{"id":2,"title":"T","status":"todo","priority":"low"}"#).unwrap();
    assert_eq!(task.created_at(), None);
    assert_eq!(task.assigned_to(), None);
    assert_eq!(task.due_date(), None);
    assert!(task.fields().is_empty());
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
// tasks / history
// ---------------------------------------------------------------------------

fn history_row(action: &str, changes: Option<serde_json::Value>) -> TaskHistoryRow {
    TaskHistoryRow {
        id: 8,
        action: action.to_string(),
        label: None,
        changes,
        old_status: Some("todo".to_string()),
        new_status: Some("in_progress".to_string()),
        author_id: Some(42),
        author_name: Some("Jean Dupont".to_string()),
        created_at: "2026-09-15T10:04:00.000Z".to_string(),
    }
}

#[test]
fn history_labels_are_generated_from_action_and_changes() {
    let created: TaskHistoryEntry = history_row("task_created", None).into();
    assert_eq!(created.id, "history-8");
    assert_eq!(created.label, "Task created");
    assert_eq!(created.author.id, "user-42");
    assert_eq!(created.author.name, "Jean Dupont");

    let status: TaskHistoryEntry = history_row("status_changed", None).into();
    assert_eq!(status.label, "Status changed: todo → in_progress");

    let updated: TaskHistoryEntry = history_row(
        "task_updated",
        Some(serde_json::json!({
            "fields": { "from": [], "to": [] },
            "title": { "from": "A", "to": "B" }
        })),
    )
    .into();
    assert_eq!(updated.label, "Task updated: title, fields");
}

#[test]
fn history_keeps_a_migrated_label_and_signs_system_writes() {
    let mut row = history_row("task_updated", None);
    row.label = Some("Budget revised".to_string());
    row.author_id = None;
    row.author_name = None;
    let entry: TaskHistoryEntry = row.into();
    assert_eq!(entry.label, "Budget revised");
    assert_eq!(entry.author.id, "system");
    assert_eq!(entry.author.name, "System");
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
    let view = GetProjectUsersQueryView::new(15, 100, 0);
    assert_eq!(view.project_id(), 15);
    assert_eq!(view.query_params().len(), 3);
    assert!(view.query_sql().contains("WHERE pm.project_id = $1"));
}

#[test]
fn remove_user_from_project_view_accessors() {
    let view = RemoveUserFromProjectQueryView::new(2, 9);
    assert_eq!(view.project_id(), 2);
    assert_eq!(view.user_id(), 9);
    assert_eq!(view.query_params().len(), 2);
    assert!(view.query_sql().contains("DELETE FROM project_members"));
}

#[test]
fn patch_task_body_with_only_a_status_is_detected() {
    use project_api::endpoints::v1::projects::project_id::tasks::task_id::patch::view::PatchTaskView;

    let status_only: PatchTaskView = serde_json::from_str(r#"{"status":"Completed"}"#).unwrap();
    let with_title: PatchTaskView =
        serde_json::from_str(r#"{"status":"Completed","name":"Renommée"}"#).unwrap();
    let clearing_assignee: PatchTaskView =
        serde_json::from_str(r#"{"status":"Completed","assigned_to":null}"#).unwrap();

    assert!(status_only.only_changes_status());
    assert!(!with_title.only_changes_status());
    assert!(!clearing_assignee.only_changes_status());
    assert_eq!(clearing_assignee.assigned_to, Some(None));
}
