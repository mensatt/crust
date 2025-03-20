use async_graphql::{Context, InputObject, Result};
use diesel::prelude::*;

use crate::{
    db::{
        conn::DbPool,
        models::tag::{DbTag, TagPriority},
    },
    graphql::queries::GqlTag,
};

#[derive(Debug, InputObject, Insertable)]
#[diesel(table_name = crate::schema::tags)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CreateTagInput {
    pub key: String,
    pub name: String,
    pub description: String,
    pub short_name: Option<String>,
    pub priority: TagPriority,
    pub is_allergy: Option<bool>,
}

#[derive(Debug, InputObject, AsChangeset)]
#[diesel(table_name = crate::schema::tags)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UpdateTagInput {
    pub key: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub short_name: Option<String>,
    pub priority: Option<TagPriority>,
    pub is_allergy: Option<bool>,
}

#[derive(Default)]
pub struct TagMutations;

#[async_graphql::Object]
impl TagMutations {
    async fn create_tag(&self, ctx: &Context<'_>, input: CreateTagInput) -> Result<GqlTag> {
        use crate::schema::tags;

        // Get DB connection
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        // Add tag
        let results: DbTag = diesel::insert_into(tags::table)
            .values(&input)
            .get_result(conn)
            .expect("Error saving new tag");

        Ok(results.into())
    }

    async fn update_tag(&self, ctx: &Context<'_>, input: UpdateTagInput) -> Result<GqlTag> {
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
                .select(DbTag::as_select())
                .first(conn)
        });

        Ok(results.map(|db_tag: DbTag| db_tag.into())?)
    }
}
