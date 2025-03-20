use async_graphql::{Context, Result};
use diesel::prelude::*;

use crate::db::{
    conn::DbPool,
    models::tag::{CreateTagInput, Tag, UpdateTagInput},
};

#[derive(Default)]
pub struct TagMutations;

#[async_graphql::Object]
impl TagMutations {
    async fn create_tag(&self, ctx: &Context<'_>, input: CreateTagInput) -> Result<Tag> {
        use crate::schema::tags;

        // Get DB connection
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        // Add tag
        let results = diesel::insert_into(tags::table)
            .values(&input)
            .get_result(conn)
            .expect("Error saving new tag");

        Ok(results)
    }

    async fn update_tag(&self, ctx: &Context<'_>, input: UpdateTagInput) -> Result<Tag> {
        use crate::schema::tags;

        // Get DB connection
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        // Create query to update the given tag
        let query = diesel::update(tags::table)
            .filter(tags::key.eq(&input.key))
            .set(&input);

        // Try to update, map empty changeset to None (instead of Error)
        let pot_empty_changeset = query
            .get_result(conn)
            .optional_empty_changeset()
            .expect("Error while updating");

        // Get tag from DB if changeset was empty (== no changes should be made to object)
        let results = pot_empty_changeset.map(Ok).unwrap_or_else(|| {
            // Fallback query that returns the tag as it is stored in the databse
            tags::table
                .filter(tags::key.eq(&input.key))
                .select(Tag::as_select())
                .first(conn)
        });

        Ok(results?)
    }
}
