use std::collections::HashMap;

use async_graphql::dataloader::Loader;
use diesel::prelude::*;

use crate::db::{conn::DbPool, models::tag::DbTag};
use crate::schema::{occurrences_tags, tags};

pub struct TagLoader {
    pub pool: DbPool,
}

impl Loader<uuid::Uuid> for TagLoader {
    type Value = Vec<DbTag>;
    type Error = async_graphql::Error;

    async fn load(
        &self,
        keys: &[uuid::Uuid],
    ) -> Result<HashMap<uuid::Uuid, Self::Value>, Self::Error> {
        // println!("Executing tag loader for {:?} elements", keys.len());

        // Get DB connection
        let conn = &mut self.pool.get().unwrap();

        // Resolve tag(s) for the given occurrence id(s)
        let rows = occurrences_tags::table
            .filter(occurrences_tags::occurrence.eq_any(keys))
            .inner_join(tags::table)
            .select((occurrences_tags::occurrence, DbTag::as_select()))
            .load::<(uuid::Uuid, DbTag)>(conn)?;

        // Add tags to their occurrences
        let mut map = HashMap::new();
        for (occ_id, tag) in rows {
            map.entry(occ_id).or_insert_with(Vec::new).push(tag);
        }
        Ok(map)
    }
}
