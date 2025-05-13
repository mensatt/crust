use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};

use crate::config::DatabaseConfig;

// Type alias to simplify code
pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub fn create_db_pool(db_config: &DatabaseConfig) -> DbPool {
    let manager = ConnectionManager::<PgConnection>::new(db_config.url());
    Pool::builder()
        .build(manager)
        .expect("Unable to create pool")
}
