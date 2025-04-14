use std::collections::HashMap;

use async_graphql::dataloader::Loader;
use diesel::prelude::*;

use crate::db::{conn::DbPool, models::location::DbLocation};
use crate::schema::locations;

pub struct LocationLoader {
    pub pool: DbPool,
}

impl Loader<uuid::Uuid> for LocationLoader {
    type Value = DbLocation;
    type Error = async_graphql::Error;

    async fn load(
        &self,
        keys: &[uuid::Uuid],
    ) -> Result<HashMap<uuid::Uuid, Self::Value>, Self::Error> {
        // println!("Executing location loader for {:?} elements", keys.len());

        // Get DB connection
        let conn = &mut self.pool.get().unwrap();

        // Resolve location(s) with the given id(s)
        let results = locations::table
            .filter(locations::id.eq_any(keys))
            .load::<DbLocation>(conn)?;

        Ok(results.into_iter().map(|l| (l.id, l)).collect())
    }
}
