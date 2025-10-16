use chrono::TimeZone;

use crate::db::conn::{DbConn, DbPool};
use crate::graphql::error::GqlApiError;

// Alias that renames chrono's NaiveDate to Date in the GraphQL schema
#[derive(async_graphql::NewType, Debug, Clone, Copy)]
#[graphql(name = "Date")]
pub struct GqlDate(chrono::NaiveDate);

// To convert back to DateTime<UTC> (which is what is stored in database)
impl From<GqlDate> for chrono::DateTime<chrono::Utc> {
    fn from(value: GqlDate) -> Self {
        let naive_date_time = &value
            .0
            .and_hms_opt(0, 0, 0)
            .expect("Unable to convert GqlDate to chrono DateTime");
        chrono::Utc.from_utc_datetime(naive_date_time)
    }
}

// Alias for an RFC3339/ISO 8601 timestamp in the GraphQL schema
// Format: Microsecond precision; with "Z" for UTC timezone
// Example: 2024-02-02T08:04:08.924549Z
// Note: This format was chosen to be backwards-compatible with the previous API implementation
#[derive(async_graphql::NewType, Debug, Clone)]
#[graphql(name = "Timestamp")]
pub struct GqlTimestamp(String);

// Conversion from chrono's DateTime type to the GraphQL timestamp type
impl From<chrono::DateTime<chrono::Utc>> for GqlTimestamp {
    fn from(value: chrono::DateTime<chrono::Utc>) -> Self {
        value
            .to_rfc3339_opts(chrono::SecondsFormat::Micros, true)
            .into()
    }
}

// Helper function to get a connection via the schema context with appropriate error conversion
pub fn get_conn_from_ctx(ctx: &async_graphql::Context) -> async_graphql::Result<DbConn> {
    let pool = ctx
        .data::<DbPool>()
        .map_err(|e| GqlApiError::internal("Unable to get pool from context.", e.message))?;

    Ok(get_conn_from_pool(pool)?)
}

pub fn get_conn_from_pool(pool: &DbPool) -> async_graphql::Result<DbConn> {
    let conn = pool
        .get()
        .map_err(|e| GqlApiError::internal("Unable to get connection from pool.", e.to_string()))?;
    Ok(conn)
}
