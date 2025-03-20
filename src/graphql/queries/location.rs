use async_graphql::Context;
use diesel::prelude::*;

use crate::db::{conn::DbPool, models::location::Location};

#[derive(Default)]
pub struct LocationQueries;

#[async_graphql::Object]
impl LocationQueries {
    // TODO: Filter
    async fn locations(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Location>> {
        use crate::schema::locations::dsl::*;

        // Get DB conn
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        // Construct and execute query
        let query = locations.select(Location::as_select());
        let results = query.load(conn).expect("Error loading locations");
        Ok(results)
    }
}
