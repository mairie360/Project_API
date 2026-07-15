use mairie360_api_lib::database::db_interface::DatabaseQueryView;
use mairie360_api_lib::database::errors::DatabaseError;
use sqlx::PgPool;

use crate::database::users::remove_user_from_project::view::RemoveUserFromProjectQueryView;

pub async fn remove_user_from_project_query(
    view: RemoveUserFromProjectQueryView,
    pool: PgPool, // ou &PgPool selon ta configuration globale
) -> Result<u64, DatabaseError> {
    // 1. On exécute la requête configurée dans ta view
    let rows_affected = sqlx::query(&view.get_request())
        .bind(view.project_id() as i32)
        .bind(view.user_id() as i32)
        .execute(&pool)
        .await?
        .rows_affected();

    Ok(rows_affected)
}
