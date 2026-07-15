use actix_web::web;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

use crate::{
    database::tasks::get_project_tasks::view::DynamicTaskField,
    endpoints::v1::projects::project_id::{
        get::view::{TaskPriority, TaskStatus},
        tasks::post::endpoint::CreateTaskError,
    },
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
    fields: Vec<DynamicTaskField>,
}

impl CreateTaskView {
    pub fn new(name: String, fields: Vec<DynamicTaskField>) -> Self {
        Self {
            name,
            description: None,
            due_date: None,
            status: None,
            priority: None,
            assigned_to: None,
            fields,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &Option<String> {
        &self.description
    }

    pub fn due_date(&self) -> &Option<DateTime<Utc>> {
        &self.due_date
    }

    pub fn status(&self) -> Option<TaskStatus> {
        self.status.clone()
    }

    pub fn priority(&self) -> Option<TaskPriority> {
        self.priority.clone()
    }

    pub fn assigned_to(&self) -> &Option<u64> {
        &self.assigned_to
    }

    pub fn fields(&self) -> &Vec<DynamicTaskField> {
        &self.fields
    }
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
