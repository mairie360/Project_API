use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

use crate::database::project::update_status::view::ProjectStatus;

/// Met à jour le titre, la description et/ou le statut d'un projet (un champ absent est conservé).
/// Renvoie `true` si le projet existe.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateProjectQueryView {
    params: Vec<QueryParam>,
}

impl UpdateProjectQueryView {
    pub fn new(
        project_id: u64,
        title: Option<&str>,
        description: Option<&str>,
        status: Option<ProjectStatus>,
    ) -> Self {
        Self {
            params: vec![
                QueryParam::I32(project_id as i32),
                QueryParam::Text(title.unwrap_or_default().to_string()),
                QueryParam::Bool(description.is_some()),
                QueryParam::Text(description.unwrap_or_default().to_string()),
                QueryParam::Text(status.map(|s| s.to_string()).unwrap_or_default()),
            ],
        }
    }

    pub fn project_id(&self) -> u64 {
        self.params[0].as_i32() as u64
    }
}

impl ApiRequestDto for UpdateProjectQueryView {
    fn query_sql(&self) -> &'static str {
        "WITH updated AS ( \
            UPDATE projects SET \
                title = COALESCE(NULLIF($2, ''), title), \
                description = CASE WHEN $3 THEN NULLIF($4, '') ELSE description END, \
                status = COALESCE(NULLIF($5, '')::project_status, status) \
            WHERE id = $1 RETURNING id \
         ) \
         SELECT EXISTS (SELECT 1 FROM updated)"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
