use async_graphql::Context;
use diesel::prelude::*;

use crate::db::{
    conn::DbPool,
    models::dish::{Dish, DishFilter},
};

#[derive(Default)]
pub struct DishQueries;

#[async_graphql::Object]
impl DishQueries {
    async fn dishes(
        &self,
        ctx: &Context<'_>,
        filter: Option<DishFilter>,
    ) -> async_graphql::Result<Vec<Dish>> {
        use crate::schema::dishes;
        // Get DB conn
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        // Construct query
        let mut query = dishes::table.select(Dish::as_select()).into_boxed();

        // Add neccessary clauses depending on present filter values
        if let Some(f) = filter {
            if let Some(filter_dishes) = f.dishes {
                query = query.filter(dishes::id.eq_any(filter_dishes));
            }
            if let Some(filter_name_de) = f.name_de {
                query = query.filter(dishes::name_en.ilike(format!("%{}%", filter_name_de)));
            }
            if let Some(filter_name_en) = f.name_en {
                query = query.filter(dishes::name_en.ilike(format!("%{}%", filter_name_en)));
            }
        }

        // Return results
        let results = query.load(conn).expect("Error loading dishes");
        Ok(results)
    }
}
