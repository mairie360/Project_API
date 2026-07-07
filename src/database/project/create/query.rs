use mairie360_api_lib::database::db_interface::DatabaseQueryView;
use mairie360_api_lib::database::errors::DatabaseError;
use sqlx::PgPool;

use crate::database::project::create::view::CreateProjectQueryView;

pub async fn create_project_query(
    view: CreateProjectQueryView,
    pool: PgPool, // ou &PgPool selon ta configuration globale
) -> Result<i32, DatabaseError> {
    // 1. On exécute la requête configurée dans ta view
    let project_id = sqlx::query_scalar::<_, i32>(&view.get_request())
        .bind(view.title())
        .bind(view.description())
        .bind(view.owner_id() as i32)
        .fetch_one(&pool)
        .await?;

    Ok(project_id)
}
