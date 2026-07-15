use std::fmt::Display;

use mairie360_api_lib::database::db_interface::DatabaseQueryView;
use sqlx::types::Json;

use crate::database::tasks::{
    create_task::view::{TaskPriority, TaskStatus},
    get_project_tasks::view::DynamicTaskField,
};

pub struct PatchTaskQueryView {
    task_id: u64,
    title: Option<String>,
    status: Option<TaskStatus>,
    priority: Option<TaskPriority>,
    created_at: Option<chrono::NaiveDateTime>,
    assigned_to: Option<i32>,
    custom_fields: Option<Json<Vec<DynamicTaskField>>>,
}

impl PatchTaskQueryView {
    pub fn new(
        task_id: u64,
        title: Option<&str>,
        status: Option<TaskStatus>,
        priority: Option<TaskPriority>,
        created_at: Option<chrono::NaiveDateTime>,
        assigned_to: Option<i32>,
        custom_fields: Option<Json<Vec<DynamicTaskField>>>,
    ) -> Self {
        Self {
            task_id,
            title: title.map(|t| t.to_string()),
            status,
            priority,
            created_at,
            assigned_to,
            custom_fields,
        }
    }

    pub fn task_id(&self) -> u64 {
        self.task_id
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn status(&self) -> Option<TaskStatus> {
        self.status
    }

    pub fn priority(&self) -> Option<TaskPriority> {
        self.priority
    }

    pub fn created_at(&self) -> Option<&chrono::NaiveDateTime> {
        self.created_at.as_ref()
    }

    pub fn assigned_to(&self) -> Option<i32> {
        self.assigned_to
    }

    pub fn custom_fields(&self) -> Option<&Json<Vec<DynamicTaskField>>> {
        self.custom_fields.as_ref()
    }
}

impl Display for PatchTaskQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PatchTaskQueryView: task_id={}", self.task_id)
    }
}

impl DatabaseQueryView for PatchTaskQueryView {
    fn get_request(&self) -> String {
        "UPDATE tasks
         SET
            title = COALESCE($2, title),
            status = COALESCE($3, status),
            priority = COALESCE($4, priority),
            due_date = COALESCE($5, due_date),
            assigned_to = COALESCE($6, assigned_to)
         WHERE id = $1"
            .to_string()
    }
}
