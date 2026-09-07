use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

use crate::database::tasks::{
    create_task::view::{TaskPriority, TaskStatus},
    get_project_tasks::view::DynamicTaskField,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PatchTaskQueryView {
    params: Vec<QueryParam>,
}

impl PatchTaskQueryView {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        task_id: u64,
        title: Option<&str>,
        status: Option<TaskStatus>,
        priority: Option<TaskPriority>,
        due_date: Option<chrono::NaiveDateTime>,
        assigned_to: Option<i32>,
        _custom_fields: Option<Vec<DynamicTaskField>>,
    ) -> Self {
        Self {
            params: vec![
                QueryParam::I32(task_id as i32),
                QueryParam::Text(title.unwrap_or_default().to_string()),
                QueryParam::Text(status.map(|s| s.to_string()).unwrap_or_default()),
                QueryParam::Text(priority.map(|p| p.to_string()).unwrap_or_default()),
                QueryParam::Text(
                    due_date
                        .map(|d| d.format("%Y-%m-%dT%H:%M:%S").to_string())
                        .unwrap_or_default(),
                ),
                QueryParam::OptionI32(assigned_to),
            ],
        }
    }

    pub fn task_id(&self) -> u64 {
        self.params[0].as_i32() as u64
    }
}

impl ApiRequestDto for PatchTaskQueryView {
    fn query_sql(&self) -> &'static str {
        "UPDATE tasks SET \
            title = COALESCE(NULLIF($2, ''), title), \
            status = COALESCE(NULLIF($3, '')::task_status, status), \
            priority = COALESCE(NULLIF($4, '')::task_priority, priority), \
            due_date = COALESCE(NULLIF($5, '')::timestamptz, due_date), \
            assigned_to = COALESCE($6, assigned_to) \
         WHERE id = $1"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
