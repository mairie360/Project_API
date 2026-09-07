use mairie360_api_lib::database::db_interface::Database;
use mairie360_api_lib::redis::redis_interface::Redis;
use mairie360_api_lib::smart_db::SmartDatabase;

/// Construit un [`SmartDatabase`] branché sur le Postgres de test (`url`).
///
/// Le cache Redis n'est pas requis par les tests : aucune des vues de requête ne
/// déclare de `cache_key`, donc `SmartDatabase` ne touche jamais Redis. On lui
/// passe malgré tout une instance (pool paresseux, erreurs ignorées).
pub async fn get_smart_db(url: String) -> SmartDatabase {
    let db = Database::new(&url).await;
    let redis = Redis::new("redis://127.0.0.1:6379");
    SmartDatabase::new(db, redis)
}
