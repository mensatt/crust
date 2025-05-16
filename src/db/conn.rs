use anyhow::Context;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use log::info;

use crate::config::DatabaseConfig;

// Type alias to simplify code
pub type DbPool = Pool<ConnectionManager<PgConnection>>;
pub type DbConn = PooledConnection<ConnectionManager<PgConnection>>;

pub fn create_db_pool(db_config: &DatabaseConfig) -> anyhow::Result<DbPool> {
    let manager = ConnectionManager::<PgConnection>::new(db_config.url());
    let pool = Pool::builder()
        .build(manager)
        .context("Unable to establish (enough) connection(s) to database")?;
    info!("Successfully connected to database");
    Ok(pool)
}
