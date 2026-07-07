use mairie360_api_lib::database::db_interface::DatabaseQueryView;
use mairie360_api_lib::database::errors::DatabaseError;
use sqlx::PgPool;

use crate::database::users::add_user_to_project::view::AddUserToProjectQueryView;

pub async fn add_user_to_project_query(
    view: AddUserToProjectQueryView,
    pool: PgPool, // ou &PgPool selon ta configuration globale
) -> Result<(), DatabaseError> {
    // 1. On exécute la requête configurée dans ta view
    sqlx::query(&view.get_request())
        .bind(view.project_id() as i32)
        .bind(view.user_id() as i32)
        .execute(&pool)
        .await?;

    Ok(())
}
