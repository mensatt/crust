use async_graphql::{Context, InputObject, Result};
use diesel::prelude::*;
use diesel::result::Error::NotFound;

use crate::auth::AuthContext;
use crate::db::models::dish_alias::DbDishAlias;
use crate::graphql::error::GqlApiError;
use crate::graphql::queries::GqlDishAlias;
use crate::graphql::util::get_conn_from_ctx;
use crate::schema::dishes_aliases;

#[derive(Debug, InputObject, Insertable)]
#[diesel(table_name = crate::schema::dishes_aliases)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CreateDishAliasInput {
    pub alias_name: String,
    pub normalized_alias_name: String,
    pub dish: uuid::Uuid,
}

#[derive(Debug, InputObject)]
pub struct DeleteDishAliasInput {
    pub alias_name: String,
}

#[derive(Default)]
pub struct DishAliasMutations;

#[async_graphql::Object]
impl DishAliasMutations {
    async fn create_dish_alias(
        &self,
        ctx: &Context<'_>,
        input: CreateDishAliasInput,
    ) -> Result<GqlDishAlias> {
        // Require authentication for this mutation
        ctx.data::<AuthContext>()?.require_auth()?;

        // Get DB connection
        let conn = &mut get_conn_from_ctx(ctx)?;

        // Add dish alias and return it
        let results = diesel::insert_into(dishes_aliases::table)
            .values(&input)
            .get_result::<DbDishAlias>(conn)
            // NOTE: In theory .get_result() could return NotFound, but if that happens on insert
            //       something internally has gone wrong.
            .map_err(|e| {
                GqlApiError::internal("Error while inserting new dish alias", e.to_string())
            })?;

        Ok(results.into())
    }

    async fn delete_dish_alias(
        &self,
        ctx: &Context<'_>,
        input: DeleteDishAliasInput,
        // TODO: Consider other response type
        //       Number of rows affected?, id of deleted object?, Query object before deletion?
    ) -> Result<bool> {
        // Require authentication for this mutation
        ctx.data::<AuthContext>()?.require_auth()?;

        // Get DB connection
        let conn = &mut get_conn_from_ctx(ctx)?;

        diesel::delete(dishes_aliases::table)
            .filter(dishes_aliases::alias_name.eq(&input.alias_name))
            .execute(conn)
            .map_err(|e| match e {
                NotFound => GqlApiError::not_found(format!(
                    "Dish alias with name '{}' not found",
                    input.alias_name
                )),
                _ => GqlApiError::internal(
                    format!(
                        "Error while deleting dish alias with name '{}'",
                        input.alias_name
                    ),
                    e.to_string(),
                ),
            })?;

        Ok(true)
    }
}
