use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

use crate::database::tasks::get_project_tasks::view::DynamicTaskField;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Completed,
    Error,
}

impl From<String> for TaskStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "todo" => TaskStatus::Todo,
            "in_progress" => TaskStatus::InProgress,
            "completed" => TaskStatus::Completed,
            _ => TaskStatus::Error,
        }
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TaskStatus::Todo => "todo",
            TaskStatus::InProgress => "in_progress",
            TaskStatus::Completed => "completed",
            TaskStatus::Error => "error",
        };
        f.write_str(s)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Error,
}

impl From<String> for TaskPriority {
    fn from(s: String) -> Self {
        match s.as_str() {
            "low" => TaskPriority::Low,
            "medium" => TaskPriority::Medium,
            "high" => TaskPriority::High,
            _ => TaskPriority::Error,
        }
    }
}

impl std::fmt::Display for TaskPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TaskPriority::Low => "low",
            TaskPriority::Medium => "medium",
            TaskPriority::High => "high",
            TaskPriority::Error => "error",
        };
        f.write_str(s)
    }
}

/// SQL condition "user `$user` may be assigned a task of project `$project`": they own the project or
/// are one of its members, so they can see it and act on the task. Both are placeholders such as `"$7"`.
#[macro_export]
macro_rules! assignable_to_project_sql {
    ($project:literal, $user:literal) => {
        concat!(
            "(EXISTS (SELECT 1 FROM projects p WHERE p.id = ",
            $project,
            " AND p.owner_id = ",
            $user,
            "::int) OR EXISTS (SELECT 1 FROM project_members pm WHERE pm.project_id = ",
            $project,
            " AND pm.user_id = ",
            $user,
            "::int))"
        )
    };
}

/// Id returned by [`CreateTaskQueryView`] when `assigned_to` is neither the owner nor a member of the
/// project: nothing is inserted (task ids start at 1).
pub const ASSIGNEE_NOT_IN_PROJECT: i32 = 0;

/// Creates a task signed by `created_by` (`tasks.updated_by`, which the history trigger records as the
/// author of `task_created`) and returns its id, or [`ASSIGNEE_NOT_IN_PROJECT`].
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateTaskQueryView {
    params: Vec<QueryParam>,
}

impl CreateTaskQueryView {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        project_id: u64,
        created_by: u64,
        title: &str,
        description: &str,
        status: TaskStatus,
        priority: TaskPriority,
        due_date: Option<chrono::DateTime<chrono::Utc>>,
        assigned_to: Option<u64>,
        fields: &[DynamicTaskField],
    ) -> Self {
        Self {
            params: vec![
                QueryParam::I32(project_id as i32),
                QueryParam::Text(title.to_string()),
                QueryParam::Text(status.to_string()),
                QueryParam::Text(priority.to_string()),
                QueryParam::Text(due_date.map(|d| d.to_rfc3339()).unwrap_or_default()),
                QueryParam::OptionI32(assigned_to.map(|id| id as i32)),
                QueryParam::Text(serde_json::json!({ "fields": fields }).to_string()),
                QueryParam::Text(description.to_string()),
                QueryParam::I32(created_by as i32),
            ],
        }
    }

    pub fn project_id(&self) -> u64 {
        self.params[0].as_i32() as u64
    }

    pub fn title(&self) -> &str {
        self.params[1].as_text()
    }
}

impl ApiRequestDto for CreateTaskQueryView {
    fn query_sql(&self) -> &'static str {
        concat!(
            "WITH inserted AS ( \
                INSERT INTO tasks (project_id, title, status, priority, due_date, assigned_to, \
                                   custom_fields, description, updated_by) \
                SELECT $1, $2, $3::task_status, $4::task_priority, \
                       NULLIF($5, '')::timestamptz AT TIME ZONE 'UTC', $6, $7::jsonb, $8, $9 \
                WHERE $6::int IS NULL OR ",
            crate::assignable_to_project_sql!("$1", "$6"),
            " RETURNING id \
             ) \
             SELECT COALESCE((SELECT id FROM inserted), 0)"
        )
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
