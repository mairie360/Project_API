use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

use crate::database::tasks::create_task::view::{TaskPriority, TaskStatus};

/// Met à jour une tâche d'un projet. Un champ `None` est conservé ; pour `assigned_to`, `Some(None)`
/// retire l'assignation. Renvoie `true` si la tâche existe dans ce projet.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PatchTaskQueryView {
    params: Vec<QueryParam>,
}

impl PatchTaskQueryView {
    pub fn new(
        project_id: u64,
        task_id: u64,
        title: Option<&str>,
        status: Option<TaskStatus>,
        priority: Option<TaskPriority>,
        due_date: Option<chrono::DateTime<chrono::Utc>>,
        assigned_to: Option<Option<u64>>,
    ) -> Self {
        Self {
            params: vec![
                QueryParam::I32(task_id as i32),
                QueryParam::I32(project_id as i32),
                QueryParam::Text(title.unwrap_or_default().to_string()),
                QueryParam::Text(status.map(|s| s.to_string()).unwrap_or_default()),
                QueryParam::Text(priority.map(|p| p.to_string()).unwrap_or_default()),
                QueryParam::Text(due_date.map(|d| d.to_rfc3339()).unwrap_or_default()),
                QueryParam::Bool(assigned_to.is_some()),
                QueryParam::OptionI32(assigned_to.flatten().map(|id| id as i32)),
            ],
        }
    }

    pub fn task_id(&self) -> u64 {
        self.params[0].as_i32() as u64
    }

    pub fn project_id(&self) -> u64 {
        self.params[1].as_i32() as u64
    }
}

impl ApiRequestDto for PatchTaskQueryView {
    fn query_sql(&self) -> &'static str {
        // due_date est une colonne TIMESTAMP stockée en UTC.
        "WITH updated AS ( \
            UPDATE tasks SET \
                title = COALESCE(NULLIF($3, ''), title), \
                status = COALESCE(NULLIF($4, '')::task_status, status), \
                priority = COALESCE(NULLIF($5, '')::task_priority, priority), \
                due_date = COALESCE(NULLIF($6, '')::timestamptz AT TIME ZONE 'UTC', due_date), \
                assigned_to = CASE WHEN $7 THEN $8 ELSE assigned_to END, \
                updated_at = CURRENT_TIMESTAMP \
            WHERE id = $1 AND project_id = $2 RETURNING id \
         ) \
         SELECT EXISTS (SELECT 1 FROM updated)"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
