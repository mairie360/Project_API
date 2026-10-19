pub mod fixtures;

use mairie360_api_lib::database::db_interface::Database;
use mairie360_api_lib::redis::redis_interface::Redis;
use mairie360_api_lib::smart_db::SmartDatabase;
use mairie360_api_lib::test_setup::queries_setup::{get_shared_db, get_shared_db_as};

/// Postgres role Project_API connects with in every deployment (Database `api_roles.sql`,
/// MAIR-114).
pub const PROJECT_DB_ROLE: &str = "project_api";

/// Redis the test `SmartDatabase`s point at. It is not running: no query view declares a
/// `cache_key`, and without Redis the read caches (MAIR-524) are bypassed (failures are only
/// logged by the lib).
const TEST_REDIS_URL: &str = "redis://127.0.0.1:6379";

/// URL of the shared test database as [`PROJECT_DB_ROLE`] (MAIR-556): the application code under
/// test (`AppState`, query views) goes through it, so a query the role is not granted fails here
/// with `permission denied` instead of in staging. The fixtures keep the superuser URL of
/// `get_shared_db()` (see [`fixtures_db`]).
pub async fn project_db_url() -> String {
    get_shared_db_as(PROJECT_DB_ROLE).await
}

/// `SmartDatabase` on the shared test database, connected as [`PROJECT_DB_ROLE`]: the one every
/// query view under test runs through.
pub async fn get_smart_db() -> SmartDatabase {
    smart_db(&project_db_url().await).await
}

/// `SmartDatabase` on the shared test database as the **superuser**, for the test set-up only
/// (users, roles, groups, statistics...): those tables are outside Project_API's grants. Never
/// run the code under test through it.
pub async fn fixtures_db() -> SmartDatabase {
    let (_container, url) = get_shared_db().await;
    smart_db(url).await
}

async fn smart_db(url: &str) -> SmartDatabase {
    let db = Database::new(url).await;
    let redis = Redis::new(TEST_REDIS_URL);
    SmartDatabase::new(db, redis)
}
