use async_graphql::{Context, Result};
use diesel::prelude::*;

use crate::db::{conn::DbPool, models::tag::Tag};

#[derive(Default)]
pub struct TagQueries;

#[async_graphql::Object]
impl TagQueries {
    async fn tags(&self, ctx: &Context<'_>) -> Result<Vec<Tag>> {
        use crate::schema::tags::dsl::*;

        // Get DB conn
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        // Construct and execute query
        let query = tags.select(Tag::as_select());
        let results = query.load(conn).expect("Error loading tags");
        Ok(results)
    }
}
