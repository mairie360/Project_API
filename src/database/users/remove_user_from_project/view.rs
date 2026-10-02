use mairie360_api_lib::database::db_interface::{
    id_from_sql, id_to_sql, ApiRequestDto, QueryParam,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RemoveUserFromProjectQueryView {
    params: Vec<QueryParam>,
}

impl RemoveUserFromProjectQueryView {
    pub fn new(project_id: u64, user_id: u64) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(project_id)),
                QueryParam::I32(id_to_sql(user_id)),
            ],
        }
    }

    pub fn project_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }

    pub fn user_id(&self) -> u64 {
        id_from_sql(self.params[1].as_i32())
    }
}

impl ApiRequestDto for RemoveUserFromProjectQueryView {
    fn query_sql(&self) -> &'static str {
        "DELETE FROM project_members WHERE project_id = $1 AND user_id = $2"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Unassigns the tasks of the project that are assigned to a removed member, signed by the caller
/// (`tasks.updated_by`, recorded by the history trigger): a task can only be assigned to the owner
/// or a member of its project.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UnassignMemberTasksQueryView {
    params: Vec<QueryParam>,
}

impl UnassignMemberTasksQueryView {
    pub fn new(project_id: u64, user_id: u64, updated_by: u64) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(project_id)),
                QueryParam::I32(id_to_sql(user_id)),
                QueryParam::I32(id_to_sql(updated_by)),
            ],
        }
    }
}

impl ApiRequestDto for UnassignMemberTasksQueryView {
    fn query_sql(&self) -> &'static str {
        // The owner stays allowed: only a user who is no longer the owner nor a member loses
        // their tasks.
        "UPDATE tasks SET assigned_to = NULL, updated_by = $3 \
         WHERE project_id = $1 AND assigned_to = $2 \
           AND NOT EXISTS (SELECT 1 FROM projects p WHERE p.id = $1 AND p.owner_id = $2)"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
