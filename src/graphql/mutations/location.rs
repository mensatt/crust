use async_graphql::{Context, InputObject, Result};
use diesel::prelude::*;
use diesel::result::Error::NotFound;

use crate::auth::AuthContext;
use crate::db::models::location::DbLocation;
use crate::graphql::error::GqlApiError;
use crate::graphql::queries::GqlLocation;
use crate::graphql::util::get_conn_from_ctx;
use crate::schema::locations;

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
        // Require authentication for this mutation
        ctx.data::<AuthContext>()?.require_auth()?;

        // Get DB connection
        let conn = &mut get_conn_from_ctx(ctx)?;

        // Add location and return it
        let results: DbLocation = diesel::insert_into(locations::table)
            .values(&input)
            .get_result(conn)
            // NOTE: In theory .get_result() could return NotFound, but if that happens on insert
            //       something internally has gone wrong.
            .map_err(|e| {
                GqlApiError::internal("Error while inserting new location", e.to_string())
            })?;

        Ok(results.into())
    }

    async fn update_location(
        &self,
        ctx: &Context<'_>,
        input: UpdateLocationInput,
    ) -> Result<GqlLocation> {
        // Require authentication for this mutation
        ctx.data::<AuthContext>()?.require_auth()?;

        // Get DB connection
        let conn = &mut get_conn_from_ctx(ctx)?;

        // Try to update, map empty changeset to None (instead of Error)
        let pot_empty_changeset = diesel::update(locations::table)
            .filter(locations::id.eq(input.id))
            .set(&input)
            .get_result(conn)
            .optional_empty_changeset()
            .map_err(|e| match e {
                NotFound => {
                    GqlApiError::not_found(format!("Location with ID '{}' not found", input.id))
                }
                _ => GqlApiError::internal(
                    format!("Error while updating location with ID '{}'", input.id),
                    e.to_string(),
                ),
            })?;

        // Use non-empty changeset if present and fall back to querying otherwise
        let result = match pot_empty_changeset {
            Some(location) => location,
            // Fallback query that returns the location as it is stored in the database
            None => locations::table
                .filter(locations::id.eq(input.id))
                .select(DbLocation::as_select())
                .first(conn)
                .map_err(|e| match e {
                    NotFound => {
                        GqlApiError::not_found(format!("Location with ID '{}' not found", input.id))
                    }
                    _ => GqlApiError::internal(
                        format!("Error while updating location with ID '{}'", input.id),
                        e.to_string(),
                    ),
                })?,
        };

        Ok(result.into())
    }
}
