pub mod fixtures;

use mairie360_api_lib::database::db_interface::Database;
use mairie360_api_lib::redis::redis_interface::Redis;
use mairie360_api_lib::smart_db::SmartDatabase;
use mairie360_api_lib::test_setup::isolated::{isolated_db, IsolatedDb};
use std::cell::RefCell;
use std::rc::Rc;

thread_local! {
    /// The database of the test running on this thread (MAIR-1002). libtest runs every test on a
    /// thread of its own and the tests run on one thread (`#[tokio::test]`, `#[actix_web::test]`),
    /// so the first helper a test calls creates its database and the end of the thread drops it.
    static OWN_DB: RefCell<Option<Rc<IsolatedDb>>> = const { RefCell::new(None) };
}

/// The database of the calling test: a copy of the seeded template, created on the first call and
/// dropped when the test ends. No test reads what another one writes. Every helper below goes
/// through it, so the fixtures, the `AppState` and the query views of a test see the same rows.
pub async fn own_db() -> Rc<IsolatedDb> {
    if let Some(db) = OWN_DB.with(|cell| cell.borrow().clone()) {
        return db;
    }
    let db = Rc::new(isolated_db().await);
    OWN_DB.with(|cell| *cell.borrow_mut() = Some(db.clone()));
    db
}

/// Postgres role Project_API connects with in every deployment (Database `api_roles.sql`,
/// MAIR-114).
pub const PROJECT_DB_ROLE: &str = "project_api";

/// Redis the test `SmartDatabase`s point at. It is not running: no query view declares a
/// `cache_key`, and without Redis the read caches (MAIR-524) are bypassed (failures are only
/// logged by the lib).
const TEST_REDIS_URL: &str = "redis://127.0.0.1:6379";

/// URL of the database of the calling test as [`PROJECT_DB_ROLE`] (MAIR-556): the application code
/// under test (`AppState`, query views) goes through it, so a query the role is not granted fails
/// here with `permission denied` instead of in staging. The fixtures keep the superuser URL (see
/// [`fixtures_db`]).
pub async fn project_db_url() -> String {
    own_db().await.url_as(PROJECT_DB_ROLE)
}

/// `SmartDatabase` on the database of the calling test, connected as [`PROJECT_DB_ROLE`]: the one every
/// query view under test runs through.
pub async fn get_smart_db() -> SmartDatabase {
    smart_db(&project_db_url().await).await
}

/// `SmartDatabase` on the database of the calling test as the **superuser**, for the test set-up only
/// (users, roles, groups, statistics...): those tables are outside Project_API's grants. Never
/// run the code under test through it.
pub async fn fixtures_db() -> SmartDatabase {
    smart_db(&own_db().await.url()).await
}

async fn smart_db(url: &str) -> SmartDatabase {
    let db = Database::new(url).await;
    let redis = Redis::new(TEST_REDIS_URL);
    SmartDatabase::new(db, redis)
}
