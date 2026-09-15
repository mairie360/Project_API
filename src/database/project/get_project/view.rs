use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

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
                QueryParam::I32(user_id as i32),
                QueryParam::I32(project_id as i32),
            ],
        }
    }

    pub fn user_id(&self) -> u64 {
        self.params[0].as_i32() as u64
    }

    pub fn project_id(&self) -> u64 {
        self.params[1].as_i32() as u64
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
