use std::collections::HashMap;

use async_graphql::dataloader::Loader;
use diesel::prelude::*;
use log::debug;

use crate::db::{conn::DbPool, models::image::DbImage};
use crate::graphql::error::GqlApiError;
use crate::graphql::util::get_conn_from_pool;
use crate::schema::images;

pub struct ImageLoader {
    pub pool: DbPool,
}

impl Loader<uuid::Uuid> for ImageLoader {
    type Value = Vec<DbImage>;
    type Error = async_graphql::Error;

    async fn load(
        &self,
        keys: &[uuid::Uuid],
    ) -> Result<HashMap<uuid::Uuid, Self::Value>, Self::Error> {
        debug!("Loading {:?} elements", keys.len());

        // Get DB connection
        let conn = &mut get_conn_from_pool(&self.pool)?;

        // Resolve image(s) for the given review id(s)
        let rows = images::table
            .filter(images::review.eq_any(keys))
            .select(DbImage::as_select())
            .load::<DbImage>(conn)
            .map_err(|e| GqlApiError::internal("Error while loading images", e.to_string()))?;

        // Group images by their review id
        let mut map: HashMap<uuid::Uuid, Vec<DbImage>> = HashMap::new();
        for image in rows {
            map.entry(image.review).or_default().push(image);
        }
        Ok(map)
    }
}
