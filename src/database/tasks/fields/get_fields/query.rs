use mairie360_api_lib::database::db_interface::DatabaseQueryView;
use mairie360_api_lib::database::errors::DatabaseError;
use sqlx::PgPool;

use crate::database::tasks::fields::get_fields::view::{
    CustomeFieldsQueryResultView, GetTaskFieldsQueryView,
};

pub async fn get_task_custom_fields_query(
    view: GetTaskFieldsQueryView,
    pool: PgPool, // ou &PgPool selon ta configuration globale
) -> Result<CustomeFieldsQueryResultView, DatabaseError> {
    // 1. On exécute la requête configurée dans ta view
    let fields = sqlx::query_as::<_, CustomeFieldsQueryResultView>(&view.get_request())
        .bind(view.id() as i32)
        .fetch_one(&pool)
        .await?;

    Ok(fields)
}
