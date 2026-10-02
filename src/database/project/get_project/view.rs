use mairie360_api_lib::database::db_interface::{
    id_from_sql, id_to_sql, ApiRequestDto, QueryParam,
};

/// Le projet `project_id` s'il est visible par l'utilisateur (aucune ligne sinon, ou s'il n'existe pas).
/// Les lignes sont des `get_projects::view::ProjectView`.
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
                SELECT p.id, p.title, p.description, p.status \
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
