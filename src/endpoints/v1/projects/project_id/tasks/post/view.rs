use actix_web::web;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

use crate::endpoints::v1::projects::project_id::{
    get::view::{TaskPriority, TaskStatus},
    tasks::{get::view::TaskFieldType, post::endpoint::CreateTaskError},
};

#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct CreateTaskView {
    name: String,
    description: Option<String>,
    #[schema(value_type = Option<String>, format = DateTime)]
    due_date: Option<DateTime<Utc>>,
    status: Option<TaskStatus>,
    priority: Option<TaskPriority>,
    assigned_to: Option<u64>,
    fields: Vec<TaskFieldType>,
}

impl TryFrom<web::Json<CreateTaskView>> for CreateTaskView {
    type Error = CreateTaskError;

    fn try_from(params: web::Json<CreateTaskView>) -> Result<CreateTaskView, Self::Error> {
        Ok(params.into_inner())
    }
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct CreateTaskResultView {
    pub task_id: u64,
    pub name: String,
    pub description: Option<String>,
}
