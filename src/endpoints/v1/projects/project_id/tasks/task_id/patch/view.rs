use crate::endpoints::v1::projects::project_id::{
    get::view::{TaskPriority, TaskStatus},
    tasks::{get::view::TaskFieldType, task_id::patch::endpoint::PatchTaskError},
};
use actix_web::web;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

#[derive(Debug, serde::Deserialize, ToSchema)]
struct PatchField {
    id: u64,
    field_type: TaskFieldType,
}

#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct PatchTaskView {
    name: Option<String>,
    description: Option<String>,
    status: Option<TaskStatus>,
    priority: Option<TaskPriority>,
    #[schema(value_type = Option<String>, format = DateTime)]
    due_date: Option<DateTime<Utc>>,
    assigned_to: Option<u64>,
    fields: Option<Vec<PatchField>>,
}

impl TryFrom<web::Json<PatchTaskView>> for PatchTaskView {
    type Error = PatchTaskError;

    fn try_from(params: web::Json<PatchTaskView>) -> Result<PatchTaskView, Self::Error> {
        Ok(params.into_inner())
    }
}
