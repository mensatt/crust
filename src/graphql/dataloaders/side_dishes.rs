use std::collections::HashMap;

use async_graphql::dataloader::Loader;
use diesel::prelude::*;

use crate::db::{conn::DbPool, models::dish::DbDish};
use crate::schema::{dishes, occurrences_side_dishes};

pub struct SideDishLoader {
    pub pool: DbPool,
}

impl Loader<uuid::Uuid> for SideDishLoader {
    type Value = Vec<DbDish>;
    type Error = async_graphql::Error;

    async fn load(
        &self,
        keys: &[uuid::Uuid],
    ) -> Result<HashMap<uuid::Uuid, Self::Value>, Self::Error> {
        // println!("Executing side dish loader for {:?} elements", keys.len());

        // Get DB connection
        let conn = &mut self.pool.get().unwrap();

        // Resolve side dish(es) for the given occurrence id(s)
        let rows = occurrences_side_dishes::table
            .filter(occurrences_side_dishes::occurrence.eq_any(keys))
            .inner_join(dishes::table)
            .select((occurrences_side_dishes::occurrence, DbDish::as_select()))
            .load::<(uuid::Uuid, DbDish)>(conn)?;

        // Add side dishes to their occurrences
        let mut map = HashMap::new();
        for (occ_id, tag) in rows {
            map.entry(occ_id).or_insert_with(Vec::new).push(tag);
        }
        Ok(map)
    }
}
