use mairie360_api_lib::database::db_interface::{
    id_from_sql, id_to_sql, ApiRequestDto, QueryParam,
};

/// One task of a project, read alone (MAIR-474): the BFF checks the rights on a task without
/// reading every task of its project. No row when the task is not in this project. Read with
/// `fetch_all::<Task, _>` (see `get_project_tasks::view::Task`).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetTaskQueryView {
    params: Vec<QueryParam>,
}

impl GetTaskQueryView {
    pub fn new(project_id: u64, task_id: u64) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(project_id)),
                QueryParam::I32(id_to_sql(task_id)),
            ],
        }
    }

    pub fn project_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }

    pub fn task_id(&self) -> u64 {
        id_from_sql(self.params[1].as_i32())
    }
}

impl ApiRequestDto for GetTaskQueryView {
    fn query_sql(&self) -> &'static str {
        // Same columns as a page of `GetProjectTasksQueryView`, so the row reads as a `Task`.
        "SELECT to_jsonb(t) FROM ( \
            SELECT id, title, description, status, priority, created_at, assigned_to, \
                   due_date AT TIME ZONE 'UTC' AS due_date, \
                   archived_at AT TIME ZONE 'UTC' AS archived_at, \
                   COALESCE(custom_fields, '{}'::jsonb) AS custom_fields \
            FROM tasks WHERE project_id = $1 AND id = $2 \
         ) t"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
