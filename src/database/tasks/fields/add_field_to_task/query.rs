use mairie360_api_lib::database::db_interface::DatabaseQueryView;
use mairie360_api_lib::database::errors::DatabaseError;
use sqlx::PgPool;

use crate::database::tasks::fields::add_field_to_task::view::AddFieldToTaskQueryView;

pub async fn add_field_to_task_query(
    view: AddFieldToTaskQueryView,
    pool: PgPool,
) -> Result<u64, DatabaseError> {
    let result = sqlx::query(&view.get_request())
        .bind(view.task_id() as i32)
        .bind(view.custom_fields()) // SQLx gère le mapping de Json vers JSONB
        .execute(&pool)
        .await?;

    Ok(result.rows_affected())
}
