use async_graphql::{Context, InputObject, Result};
use diesel::prelude::*;
use diesel::result::Error::NotFound;

use crate::auth::AuthContext;
use crate::db::models::tag::{DbTag, TagPriority};
use crate::graphql::error::GqlApiError;
use crate::graphql::queries::GqlTag;
use crate::graphql::util::get_conn_from_ctx;
use crate::schema::tags;

#[derive(Debug, InputObject, Insertable)]
#[diesel(table_name = crate::schema::tags)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CreateTagInput {
    pub key: String,
    pub name: String,
    pub description: String,
    pub short_name: Option<String>,
    pub priority: Option<TagPriority>,
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
        // Require authentication for this mutation
        ctx.data::<AuthContext>()?.require_auth()?;

        // Get DB connection
        let conn = &mut get_conn_from_ctx(ctx)?;

        // Add tag and return it
        let results: DbTag = diesel::insert_into(tags::table)
            .values(&input)
            .get_result(conn)
            // NOTE: In theory .get_result() could return NotFound, but if that happens on insert
            //       something internally has gone wrong.
            .map_err(|e| GqlApiError::internal("Error while inserting new tag", e.to_string()))?;

        Ok(results.into())
    }

    async fn update_tag(&self, ctx: &Context<'_>, input: UpdateTagInput) -> Result<GqlTag> {
        // Require authentication for this mutation
        ctx.data::<AuthContext>()?.require_auth()?;

        // Get DB connection
        let conn = &mut get_conn_from_ctx(ctx)?;

        // Try to update, map empty changeset to None (instead of Error)
        let pot_empty_changeset = diesel::update(tags::table)
            .filter(tags::key.eq(&input.key))
            .set(&input)
            .get_result::<DbTag>(conn)
            .optional_empty_changeset()
            .map_err(|e| match e {
                NotFound => GqlApiError::not_found(format!("Tag '{}' not found", input.key)),
                _ => GqlApiError::internal(
                    format!("Error while updating tag '{}'", input.key),
                    e.to_string(),
                ),
            })?;

        // Use non-empty changeset if present and fall back to querying otherwise
        let result = match pot_empty_changeset {
            Some(tag) => tag,
            // Fallback query that returns the tag as it is stored in the database
            None => tags::table
                .filter(tags::key.eq(&input.key))
                .select(DbTag::as_select())
                .first(conn)
                .map_err(|e| match e {
                    NotFound => GqlApiError::not_found(format!("Tag '{}' not found", input.key)),
                    _ => GqlApiError::internal(
                        format!("Error while updating tag '{}'", input.key),
                        e.to_string(),
                    ),
                })?,
        };

        Ok(result.into())
    }
}
