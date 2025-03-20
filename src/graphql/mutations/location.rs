use async_graphql::Context;
use diesel::prelude::*;

use crate::db::{
    conn::DbPool,
    models::location::{CreateLocationInput, Location, UpdateLocationInput},
};

#[derive(Default)]
pub struct LocationMutations;

#[async_graphql::Object]
impl LocationMutations {
    async fn create_location(
        &self,
        ctx: &Context<'_>,
        input: CreateLocationInput,
    ) -> async_graphql::Result<Location> {
        use crate::schema::locations;

        // Get DB connection
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        // Add location
        let results = diesel::insert_into(locations::table)
            .values(&input)
            .get_result(conn)
            .expect("Error saving new tag");

        Ok(results)
    }

    async fn update_location(
        &self,
        ctx: &Context<'_>,
        input: UpdateLocationInput,
    ) -> async_graphql::Result<Location> {
        use crate::schema::locations;

        // Get DB connection
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        // Create query to update the given tag
        let query = diesel::update(locations::table)
            .filter(locations::id.eq(input.id))
            .set(&input);

        // Try to update, map empty changeset to None (instead of Error)
        let pot_empty_changeset = query
            .get_result(conn)
            .optional_empty_changeset()
            .expect("Error while updating");

        // Get tag from DB if changeset was empty (== no changes should be made to object)
        let results = pot_empty_changeset.map(Ok).unwrap_or_else(|| {
            // Fallback query that returns the tag as it is stored in the databse
            locations::table
                .filter(locations::id.eq(input.id))
                .select(Location::as_select())
                .first(conn)
        });

        Ok(results?)
    }
}
