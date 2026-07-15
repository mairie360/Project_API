use mairie360_api_lib::database::db_interface::DatabaseQueryView;
use mairie360_api_lib::database::errors::DatabaseError;
use sqlx::PgPool;

use crate::database::project::update_status::view::UpdateProjectStatusQueryView;

pub async fn update_project_status_query(
    view: UpdateProjectStatusQueryView,
    pool: PgPool, // ou &PgPool selon ta configuration globale
) -> Result<u64, DatabaseError> {
    // 1. On exécute la requête configurée dans ta view
    let project_id = sqlx::query(&view.get_request())
        .bind(view.status())
        .bind(view.project_id() as i32)
        .execute(&pool)
        .await?
        .rows_affected();

    Ok(project_id)
}
