use mairie360_api_lib::database::db_interface::DatabaseQueryView;
use mairie360_api_lib::database::errors::DatabaseError;
use sqlx::PgPool;

use crate::database::users::get_project_users::view::GetProjectUsersQueryView;

pub async fn get_project_users_query(
    view: GetProjectUsersQueryView,
    pool: PgPool, // ou &PgPool selon ta configuration globale
) -> Result<Vec<i32>, DatabaseError> {
    // 1. On exécute la requête configurée dans ta view
    let result = sqlx::query_scalar::<_, i32>(&view.get_request())
        .bind(view.project_id() as i32)
        .fetch_all(&pool)
        .await?;

    Ok(result)
}
