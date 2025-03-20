use async_graphql::{Context, InputObject, Result};
use diesel::prelude::*;

use crate::db::{conn::DbPool, models::location::DbLocation};
use crate::graphql::queries::GqlLocation;

#[derive(Debug, InputObject, Insertable)]
#[diesel(table_name = crate::schema::locations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CreateLocationInput {
    pub external_id: i64,
    pub name: String,
    pub visible: Option<bool>,
}

#[derive(Debug, InputObject, AsChangeset)]
#[diesel(table_name = crate::schema::locations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UpdateLocationInput {
    pub id: uuid::Uuid,
    pub external_id: Option<i64>,
    pub name: Option<String>,
    pub visible: Option<bool>,
}

#[derive(Default)]
pub struct LocationMutations;

#[async_graphql::Object]
impl LocationMutations {
    async fn create_location(
        &self,
        ctx: &Context<'_>,
        input: CreateLocationInput,
    ) -> Result<GqlLocation> {
        use crate::schema::locations;

        // Get DB connection
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        // Add location
        let results: DbLocation = diesel::insert_into(locations::table)
            .values(&input)
            .get_result(conn)
            .expect("Error saving new tag");

        Ok(results.into())
    }

    async fn update_location(
        &self,
        ctx: &Context<'_>,
        input: UpdateLocationInput,
    ) -> Result<GqlLocation> {
        use crate::schema::locations;

        // Get DB connection
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        // Create query to update the given location
        let query = diesel::update(locations::table)
            .filter(locations::id.eq(input.id))
            .set(&input);

        // Try to update, map empty changeset to None (instead of Error)
        let pot_empty_changeset = query
            .get_result(conn)
            .optional_empty_changeset()
            .expect("Error while updating");

        // Get location from DB if changeset was empty (== no changes should be made to object)
        let results = pot_empty_changeset.map(Ok).unwrap_or_else(|| {
            // Fallback query that returns the location as it is stored in the databse
            locations::table
                .filter(locations::id.eq(input.id))
                .select(DbLocation::as_select())
                .first(conn)
        });

        Ok(results.map(|db_location| db_location.into())?)
    }
}
