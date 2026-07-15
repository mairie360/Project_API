use mairie360_api_lib::database::db_interface::DatabaseQueryView;
use mairie360_api_lib::database::errors::DatabaseError;
use sqlx::PgPool;

use crate::database::project::delete::view::DeleteProjectQueryView;

pub async fn delete_project_query(
    view: DeleteProjectQueryView,
    pool: PgPool, // ou &PgPool selon ta configuration globale
) -> Result<u64, DatabaseError> {
    // 1. On exécute la requête configurée dans ta view
    let rows_affected = sqlx::query(&view.get_request())
        .bind(view.project_id() as i32)
        .execute(&pool)
        .await?
        .rows_affected();

    Ok(rows_affected)
}
