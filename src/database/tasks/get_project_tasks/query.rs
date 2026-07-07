use mairie360_api_lib::database::db_interface::DatabaseQueryView;
use mairie360_api_lib::database::errors::DatabaseError;
use sqlx::PgPool;

use crate::database::tasks::get_project_tasks::view::{GetProjectTasksQueryView, Task};

pub async fn get_project_tasks_query(
    view: GetProjectTasksQueryView,
    pool: PgPool, // ou &PgPool selon ta configuration globale
) -> Result<Vec<Task>, DatabaseError> {
    // 1. On exécute la requête configurée dans ta view
    let rows = sqlx::query_as::<_, Task>(&view.get_request())
        .bind(view.project_id() as i32)
        .fetch_all(&pool)
        .await?;

    Ok(rows)
}
