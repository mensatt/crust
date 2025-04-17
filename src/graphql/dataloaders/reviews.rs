use std::collections::HashMap;

use async_graphql::dataloader::Loader;
use diesel::prelude::*;

use crate::db::{conn::DbPool, models::review::DbReview};
use crate::schema::reviews;

pub struct ReviewLoader {
    pub pool: DbPool,
}

impl Loader<uuid::Uuid> for ReviewLoader {
    type Value = DbReview;
    type Error = async_graphql::Error;

    async fn load(
        &self,
        keys: &[uuid::Uuid],
    ) -> Result<HashMap<uuid::Uuid, Self::Value>, Self::Error> {
        // println!("Executing review loader for {:?} elements", keys.len());

        // Get DB connection
        let conn = &mut self.pool.get().unwrap();

        // Resolve review(s) with the given dish id(s)
        let results = reviews::table
            .filter(reviews::id.eq_any(keys))
            .load::<DbReview>(conn)?;

        Ok(results.into_iter().map(|r| (r.id, r)).collect())
    }
}
