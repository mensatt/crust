use std::collections::HashMap;

use async_graphql::dataloader::Loader;
use diesel::prelude::*;
use log::debug;

use crate::db::{conn::DbPool, models::dish_alias::DbDishAlias};
use crate::graphql::error::GqlApiError;
use crate::graphql::util::get_conn_from_pool;
use crate::schema::dishes_aliases;

pub struct DishAliasLoader {
    pub pool: DbPool,
}

impl Loader<uuid::Uuid> for DishAliasLoader {
    type Value = Vec<DbDishAlias>;
    type Error = async_graphql::Error;

    async fn load(
        &self,
        keys: &[uuid::Uuid],
    ) -> Result<HashMap<uuid::Uuid, Self::Value>, Self::Error> {
        debug!("Loading {:?} elements", keys.len());

        // Get DB connection
        let conn = &mut get_conn_from_pool(&self.pool)?;

        // Resolve dish alias(es) for the given dish id(s)
        let rows = dishes_aliases::table
            .filter(dishes_aliases::dish.eq_any(keys))
            .select(DbDishAlias::as_select())
            .load::<DbDishAlias>(conn)
            .map_err(|e| GqlApiError::internal("Error while loading dish aliases", e.to_string()))?;

        // Group dish aliases by their dish id
        let mut map = HashMap::new();
        for alias in rows {
            map.entry(alias.dish).or_insert_with(Vec::new).push(alias);
        }
        Ok(map)
    }
}
