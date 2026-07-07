use mairie360_api_lib::database::db_interface::DatabaseQueryView;
use mairie360_api_lib::database::errors::DatabaseError;
use sqlx::PgPool;

use crate::database::tasks::create_task::view::CreateTaskQueryView;

pub async fn create_task_query(
    view: CreateTaskQueryView,
    pool: PgPool, // ou &PgPool selon ta configuration globale
) -> Result<i32, DatabaseError> {
    // 1. On exécute la requête configurée dans ta view
    let task_id = sqlx::query_scalar::<_, i32>(&view.get_request())
        .bind(view.project_id() as i32)
        .bind(view.title())
        .bind(view.status())
        .bind(view.priority())
        .bind(view.due_date())
        .bind(view.assigned_to())
        .fetch_one(&pool)
        .await?;

    Ok(task_id)
}
