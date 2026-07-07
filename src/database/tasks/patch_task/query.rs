use mairie360_api_lib::database::db_interface::DatabaseQueryView;
use mairie360_api_lib::database::errors::DatabaseError;
use sqlx::PgPool;

use crate::database::tasks::patch_task::view::PatchTaskQueryView;

pub async fn patch_task_query(
    view: PatchTaskQueryView,
    pool: PgPool, // ou &PgPool selon ta configuration globale
) -> Result<u64, DatabaseError> {
    // 1. On exécute la requête configurée dans ta view
    let rows = sqlx::query(&view.get_request())
        .bind(view.task_id() as i32)
        .bind(view.title())
        .bind(view.status())
        .bind(view.priority())
        .bind(view.created_at())
        .bind(view.assigned_to())
        .bind(view.custom_fields())
        .execute(&pool)
        .await?
        .rows_affected();

    Ok(rows)
}
