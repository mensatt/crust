use async_graphql::{Context, InputObject, Result};
use diesel::prelude::*;

use crate::db::{conn::DbPool, models::dish::DbDish};
use crate::graphql::queries::GqlDish;
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
        // Get DB connection
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

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
            .expect("Error saving new dish");

        Ok(results.into())
    }

    async fn update_dish(&self, ctx: &Context<'_>, input: UpdateDishInput) -> Result<GqlDish> {
        // Get DB connection
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        // Try to update, map empty changeset to None (instead of Error)
        let pot_empty_changeset = diesel::update(dishes::table)
            .filter(dishes::id.eq(input.id))
            .set(&input)
            .get_result(conn)
            .optional_empty_changeset()
            .expect("Error while updating dish");

        // Use non-empty changeset if present and fall back to querying otherwise
        let result = pot_empty_changeset.unwrap_or_else(|| {
            // Fallback query that returns the dish as it is stored in the databse
            dishes::table
                .filter(dishes::id.eq(input.id))
                .select(DbDish::as_select())
                .first(conn)
                .expect("Unable to get updated dish")
        });

        Ok(result.into())
    }
}
