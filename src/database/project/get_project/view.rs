use mairie360_api_lib::database::db_interface::{
    id_from_sql, id_to_sql, ApiRequestDto, QueryParam,
};

/// The project `project_id` with its linked group if the user can see it (no row otherwise, or if
/// it does not exist).
/// The rows are `get_projects::view::ProjectView`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetVisibleProjectQueryView {
    params: Vec<QueryParam>,
}

impl GetVisibleProjectQueryView {
    pub fn new(project_id: u64, user_id: u64) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(user_id)),
                QueryParam::I32(id_to_sql(project_id)),
            ],
        }
    }

    pub fn user_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }

    pub fn project_id(&self) -> u64 {
        id_from_sql(self.params[1].as_i32())
    }
}

impl ApiRequestDto for GetVisibleProjectQueryView {
    fn query_sql(&self) -> &'static str {
        concat!(
            "SELECT to_jsonb(t) FROM ( \
                SELECT p.id, p.title, p.description, p.status, p.priority, \
                    (SELECT jsonb_build_object('id', g.id, 'name', g.name) \
                     FROM project_groups pg JOIN groups g ON g.id = pg.group_id \
                     WHERE pg.project_id = p.id ORDER BY pg.linked_at, g.id LIMIT 1) AS \"group\" \
                FROM projects p \
                WHERE p.id = $2 AND ",
            crate::project_visible_to_user_sql!(),
            " ) t"
        )
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
