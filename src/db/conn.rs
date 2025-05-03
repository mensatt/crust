use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};

// Type alias to simplify code
pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub fn get_db_pool() -> DbPool {
    // TODO: Use config crate
    let database_url = "postgres://mensatt:S3cret@localhost:5432/new-mensatt";
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .build(manager)
        .expect("Unable to create pool.")
}
