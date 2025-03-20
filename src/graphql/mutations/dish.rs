use async_graphql::Context;
use diesel::prelude::*;

use crate::db::{
    conn::DbPool,
    models::dish::{CreateDishInput, Dish, UpdateDishInput},
};

#[derive(Default)]
pub struct DishMutations;

#[async_graphql::Object]
impl DishMutations {
    pub async fn create_dish(
        &self,
        ctx: &Context<'_>,
        input: CreateDishInput,
    ) -> async_graphql::Result<Dish> {
        use crate::schema::dishes;

        // Get DB connection
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        // Construct new dish
        let new_dish = Dish {
            id: uuid::Uuid::new_v4(),
            name_de: input.name_de,
            name_en: input.name_en,
        };

        // Add dish
        let results = diesel::insert_into(dishes::table)
            .values(&new_dish)
            .get_result(conn)
            .expect("Error saving new dish");

        Ok(results)
    }

    pub async fn update_dish(
        &self,
        ctx: &Context<'_>,
        input: UpdateDishInput,
    ) -> async_graphql::Result<Dish> {
        use crate::schema::dishes;

        // Get DB connection
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        // Create query to update the given dish
        let query = diesel::update(dishes::table)
            .filter(dishes::id.eq(input.id))
            .set(&input);

        // Try to update, map empty changeset to None (instead of Error)
        let pot_empty_changeset = query
            .get_result(conn)
            .optional_empty_changeset()
            .expect("Error while updating");

        // Get dish from DB if changeset was empty (== no changes should be made to object)
        let results = pot_empty_changeset.map(Ok).unwrap_or_else(|| {
            // Fallback query that returns the dish as it is stored in the databse
            dishes::table
                .filter(dishes::id.eq(input.id))
                .select(Dish::as_select())
                .first(conn)
        });

        Ok(results?)
    }
}
