use async_graphql::{Context, InputObject, Result};
use diesel::prelude::*;

use crate::auth::AuthContext;
use crate::db::{conn::DbPool, models::dish_alias::DbDishAlias};
use crate::graphql::queries::GqlDishAlias;
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
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        // Add dish alias and return it
        let results = diesel::insert_into(dishes_aliases::table)
            .values(&input)
            .get_result::<DbDishAlias>(conn)
            .expect("Error saving new dish alias");

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
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        diesel::delete(dishes_aliases::table)
            .filter(dishes_aliases::alias_name.eq(input.alias_name))
            .execute(conn)
            .expect("Failed to delete dish alias");

        Ok(true)
    }
}
