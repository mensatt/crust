use diesel::prelude::*;

pub fn establish_connection() -> PgConnection {
    // TODO: Use config crate
    let database_url = "postgres://mensatt:S3cret@localhost:6432/new-mensatt";
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}
