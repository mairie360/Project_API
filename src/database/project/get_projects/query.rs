use mairie360_api_lib::database::db_interface::DatabaseQueryView;
use mairie360_api_lib::database::errors::DatabaseError;
use sqlx::PgPool;

use crate::database::project::get_projects::view::{GetProjectsQueryView, ProjectView};

pub async fn get_projects_query(
    view: GetProjectsQueryView,
    pool: PgPool, // ou &PgPool selon ta configuration globale
) -> Result<Vec<ProjectView>, DatabaseError> {
    // 1. On exécute la requête configurée dans ta view
    let rows = sqlx::query_as::<_, ProjectView>(&view.get_request())
        .bind(view.user_id() as i32)
        .fetch_all(&pool)
        .await?;

    Ok(rows)
}
