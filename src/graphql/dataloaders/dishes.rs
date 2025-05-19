use std::collections::HashMap;

use async_graphql::dataloader::Loader;
use diesel::prelude::*;
use log::debug;

use crate::db::{conn::DbPool, models::dish::DbDish};
use crate::graphql::error::GqlApiError;
use crate::graphql::util::get_conn_from_pool;
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
        debug!("Loading {:?} elements", keys.len());

        // Get DB connection
        let conn = &mut get_conn_from_pool(&self.pool)?;

        // Resolve dish(es) with the given id(s)
        let results = dishes::table
            .filter(dishes::id.eq_any(keys))
            .load::<DbDish>(conn)
            .map_err(|e| GqlApiError::internal("Error while loading dishes", e.to_string()))?;

        Ok(results.into_iter().map(|d| (d.id, d)).collect())
    }
}
