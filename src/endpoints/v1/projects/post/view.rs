use actix_web::web;
use utoipa::ToSchema;

use crate::endpoints::v1::projects::post::endpoint::CreateProjectError;

#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct CreateProjectView {
    name: String,
    description: Option<String>,
    group_id: Option<u64>,
    // template_id: Option<u64>, ToDo
}

impl CreateProjectView {
    pub fn new(name: &str, description: Option<&str>, group_id: Option<u64>) -> Self {
        Self {
            name: name.to_string(),
            description: description.map(|d| d.to_string()),
            group_id,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn group_id(&self) -> Option<u64> {
        self.group_id
    }
}

impl TryFrom<web::Json<CreateProjectView>> for CreateProjectView {
    type Error = CreateProjectError;

    fn try_from(params: web::Json<CreateProjectView>) -> Result<CreateProjectView, Self::Error> {
        Ok(params.into_inner())
    }
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct CreateProjectResultView {
    pub project_id: u64,
}
