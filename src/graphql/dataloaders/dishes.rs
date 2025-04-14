use std::collections::HashMap;

use async_graphql::dataloader::Loader;
use diesel::prelude::*;

use crate::db::{conn::DbPool, models::dish::DbDish};
use crate::schema::dishes;

pub struct DishLoader {
    pub pool: DbPool,
}

impl Loader<uuid::Uuid> for DishLoader {
    type Value = DbDish;
    type Error = async_graphql::Error;

    async fn load(
        &self,
        keys: &[uuid::Uuid],
    ) -> Result<HashMap<uuid::Uuid, Self::Value>, Self::Error> {
        // println!("Executing dish loader for {:?} elements", keys.len());

        // Get DB connection
        let conn = &mut self.pool.get().unwrap();

        // Resolve dish(es) with the given id(s)
        let results = dishes::table
            .filter(dishes::id.eq_any(keys))
            .load::<DbDish>(conn)?;

        Ok(results.into_iter().map(|d| (d.id, d)).collect())
    }
}
