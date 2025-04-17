use std::collections::HashMap;

use async_graphql::dataloader::Loader;
use diesel::prelude::*;

use crate::db::{conn::DbPool, models::review::DbReview};
use crate::schema::{occurrences, reviews};

pub struct DishReviewLoader {
    pub pool: DbPool,
}

impl Loader<uuid::Uuid> for DishReviewLoader {
    type Value = Vec<DbReview>;
    type Error = async_graphql::Error;

    async fn load(
        &self,
        keys: &[uuid::Uuid],
    ) -> Result<HashMap<uuid::Uuid, Self::Value>, Self::Error> {
        println!("Executing dish review loader for {:?} elements", keys.len());

        // Get DB connection
        let conn = &mut self.pool.get().unwrap();

        // Resolve review(s) with the given dish id(s)
        let results = reviews::table
            .inner_join(occurrences::table)
            .filter(occurrences::dish.eq_any(keys))
            .select((DbReview::as_select(), occurrences::dish))
            .load::<(DbReview, uuid::Uuid)>(conn)?;

        // Group them by their dish
        let mut map = HashMap::new();
        for (review, dish_id) in results {
            map.entry(dish_id).or_insert_with(Vec::new).push(review);
        }

        Ok(map)
    }
}
