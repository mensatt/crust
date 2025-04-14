use std::collections::HashMap;

use async_graphql::dataloader::Loader;
use diesel::prelude::*;

use crate::db::{conn::DbPool, models::occurrence::DbOccurrence};
use crate::schema::occurrences;

pub struct OccurrenceLoader {
    pub pool: DbPool,
}

impl Loader<uuid::Uuid> for OccurrenceLoader {
    type Value = DbOccurrence;
    type Error = async_graphql::Error;

    async fn load(
        &self,
        keys: &[uuid::Uuid],
    ) -> Result<HashMap<uuid::Uuid, Self::Value>, Self::Error> {
        // println!("Executing occurrence loader for {:?} elements", keys.len());

        // Get DB connection
        let conn = &mut self.pool.get().unwrap();

        // Resolve occurrence(s) for given id(s)
        let results = occurrences::table
            .filter(occurrences::id.eq_any(keys))
            .load::<DbOccurrence>(conn)?;

        Ok(results.into_iter().map(|o| (o.id, o)).collect())
    }
}
