use std::collections::HashMap;

use async_graphql::dataloader::Loader;
use diesel::prelude::*;

use crate::db::{conn::DbPool, models::review::DbReview};
use crate::schema::reviews;

pub struct OccurrenceReviewLoader {
    pub pool: DbPool,
}

impl Loader<uuid::Uuid> for OccurrenceReviewLoader {
    type Value = Vec<DbReview>;
    type Error = async_graphql::Error;

    async fn load(
        &self,
        keys: &[uuid::Uuid],
    ) -> Result<HashMap<uuid::Uuid, Self::Value>, Self::Error> {
        // println!("Executing occurrence review loader for {:?} elements", keys.len());

        // Get DB connection
        let conn = &mut self.pool.get().unwrap();

        // Resolve review(s) with the given occurrence id(s)
        let reviews = reviews::table
            .filter(reviews::occurrence.eq_any(keys))
            .load::<DbReview>(conn)?;

        // Group them by their occurrence
        let mut map = HashMap::new();
        for review in reviews {
            map.entry(review.occurrence)
                .or_insert_with(Vec::new)
                .push(review);
        }

        Ok(map)
    }
}
