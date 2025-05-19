use std::collections::HashMap;

use async_graphql::dataloader::Loader;
use diesel::prelude::*;
use log::debug;

use crate::db::{conn::DbPool, models::location::DbLocation};
use crate::graphql::error::GqlApiError;
use crate::graphql::util::get_conn_from_pool;
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
        debug!("Loading {:?} elements", keys.len());

        // Get DB connection
        let conn = &mut get_conn_from_pool(&self.pool)?;

        // Resolve location(s) with the given id(s)
        let results = locations::table
            .filter(locations::id.eq_any(keys))
            .load::<DbLocation>(conn)
            .map_err(|e| GqlApiError::internal("Error while loading locations", e.to_string()))?;

        Ok(results.into_iter().map(|l| (l.id, l)).collect())
    }
}
