use std::collections::HashMap;

use async_graphql::dataloader::Loader;
use diesel::prelude::*;
use log::debug;

use crate::db::{conn::DbPool, models::tag::DbTag};
use crate::graphql::error::GqlApiError;
use crate::graphql::util::get_conn_from_pool;
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
        debug!("Loading {:?} elements", keys.len());

        // Get DB connection
        let conn = &mut get_conn_from_pool(&self.pool)?;

        // Resolve tag(s) for the given occurrence id(s)
        let rows = occurrences_tags::table
            .filter(occurrences_tags::occurrence.eq_any(keys))
            .inner_join(tags::table)
            .select((occurrences_tags::occurrence, DbTag::as_select()))
            .load::<(uuid::Uuid, DbTag)>(conn)
            .map_err(|e| GqlApiError::internal("Error while loading tags", e.to_string()))?;

        // Add tags to their occurrences
        let mut map = HashMap::new();
        for (occ_id, tag) in rows {
            map.entry(occ_id).or_insert_with(Vec::new).push(tag);
        }
        Ok(map)
    }
}
