use std::collections::HashMap;

use async_graphql::dataloader::Loader;
use diesel::prelude::*;
use log::debug;

use crate::db::{conn::DbPool, models::occurrence::DbOccurrence};
use crate::graphql::error::GqlApiError;
use crate::graphql::util::get_conn_from_pool;
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
        debug!("Loading {:?} elements", keys.len());

        // Get DB connection
        let conn = &mut get_conn_from_pool(&self.pool)?;

        // Resolve occurrence(s) for given id(s)
        let results = occurrences::table
            .filter(occurrences::id.eq_any(keys))
            .load::<DbOccurrence>(conn)
            .map_err(|e| GqlApiError::internal("Error while loading occurrences", e.to_string()))?;

        Ok(results.into_iter().map(|o| (o.id, o)).collect())
    }
}
