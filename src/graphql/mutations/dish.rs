use async_graphql::{Context, InputObject, Result};
use diesel::prelude::*;
use diesel::result::Error::NotFound;

use crate::auth::AuthContext;
use crate::db::models::dish::DbDish;
use crate::graphql::error::GqlApiError;
use crate::graphql::queries::GqlDish;
use crate::graphql::util::get_conn_from_ctx;
use crate::schema::dishes;

#[derive(Debug, InputObject)]
pub struct CreateDishInput {
    pub name_de: String,
    pub name_en: Option<String>,
}

#[derive(Debug, InputObject, AsChangeset)]
#[diesel(table_name = crate::schema::dishes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UpdateDishInput {
    pub id: uuid::Uuid,
    pub name_de: Option<String>,
    pub name_en: Option<String>,
}

#[derive(Default)]
pub struct DishMutations;

#[async_graphql::Object]
impl DishMutations {
    pub async fn create_dish(&self, ctx: &Context<'_>, input: CreateDishInput) -> Result<GqlDish> {
        // Require authentication for this mutation
        ctx.data::<AuthContext>()?.require_auth()?;

        // Get DB connection
        let conn = &mut get_conn_from_ctx(ctx)?;

        // Construct new dish
        let new_dish = DbDish {
            id: uuid::Uuid::new_v4(),
            name_de: input.name_de,
            name_en: input.name_en,
        };

        // Add dish and return it
        let results: DbDish = diesel::insert_into(dishes::table)
            .values(&new_dish)
            .get_result(conn)
            // NOTE: In theory .get_result() could return NotFound, but if that happens on insert
            //       something internally has gone wrong.
            .map_err(|e| GqlApiError::internal("Error while inserting new dish", e.to_string()))?;

        Ok(results.into())
    }

    async fn update_dish(&self, ctx: &Context<'_>, input: UpdateDishInput) -> Result<GqlDish> {
        // Require authentication for this mutation
        ctx.data::<AuthContext>()?.require_auth()?;

        // Get DB connection
        let conn = &mut get_conn_from_ctx(ctx)?;

        // Try to update, map empty changeset to None (instead of Error)
        let pot_empty_changeset = diesel::update(dishes::table)
            .filter(dishes::id.eq(input.id))
            .set(&input)
            .get_result(conn)
            .optional_empty_changeset()
            .map_err(|e| match e {
                NotFound => {
                    GqlApiError::not_found(format!("Dish with ID '{}' not found", input.id))
                }
                _ => GqlApiError::internal(
                    format!("Error while updating dish with ID '{}'", input.id),
                    e.to_string(),
                ),
            })?;

        // Use non-empty changeset if present and fall back to querying otherwise
        let result = match pot_empty_changeset {
            Some(dish) => dish,
            // Fallback query that returns the dish as it is stored in the database
            None => dishes::table
                .filter(dishes::id.eq(input.id))
                .select(DbDish::as_select())
                .first::<DbDish>(conn)
                .map_err(|e| match e {
                    NotFound => {
                        GqlApiError::not_found(format!("Dish with ID '{}' not found", input.id))
                    }
                    _ => GqlApiError::internal(
                        format!("Error while updating dish with ID '{}'", input.id),
                        e.to_string(),
                    ),
                })?,
        };

        Ok(result.into())
    }
}
