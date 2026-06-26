use std::collections::HashMap;

use async_graphql::dataloader::Loader;
use diesel::prelude::*;
use log::debug;

use crate::db::{conn::DbPool, models::image::DbImage};
use crate::graphql::error::GqlApiError;
use crate::graphql::util::get_conn_from_pool;
use crate::schema::{images, occurrences, reviews};

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum ImageLoaderKey {
    ByImageId { id: uuid::Uuid },
    ByReviewId { review_id: uuid::Uuid },
    ByOccurrenceId { occurrence_id: uuid::Uuid },
    ByDishId { dish_id: uuid::Uuid },
}

pub struct ImageLoader {
    pub pool: DbPool,
}

impl ImageLoader {
    fn load_by_image_id(
        &self,
        keys: &[(uuid::Uuid, ImageLoaderKey)],
        conn: &mut diesel::PgConnection,
        result_map: &mut HashMap<ImageLoaderKey, Vec<DbImage>>,
    ) -> Result<(), async_graphql::Error> {
        // Nothing to do if there are no keys
        if keys.is_empty() {
            return Ok(());
        }

        // Collect the various image ids
        let ids: Vec<uuid::Uuid> = keys.iter().map(|(id, _)| *id).collect();

        // Fetch all those ids from the DB
        let rows = images::table
            .filter(images::id.eq_any(&ids))
            .load::<DbImage>(conn)
            .map_err(|e| {
                GqlApiError::internal("Error while loading images by ID", e.to_string())
            })?;

        // Group images by their own id
        let image_map: HashMap<uuid::Uuid, DbImage> = rows.into_iter().map(|i| (i.id, i)).collect();

        // Construct a map of key => images
        for (id, key) in keys {
            let matched = image_map.get(id).cloned().into_iter().collect();
            result_map.insert(key.clone(), matched);
        }

        Ok(())
    }

    fn load_by_review_id(
        &self,
        keys: &[(uuid::Uuid, ImageLoaderKey)],
        conn: &mut diesel::PgConnection,
        result_map: &mut HashMap<ImageLoaderKey, Vec<DbImage>>,
    ) -> Result<(), async_graphql::Error> {
        // Nothing to do if there are no keys
        if keys.is_empty() {
            return Ok(());
        }

        // Collect the various review ids
        let review_ids: Vec<uuid::Uuid> = keys.iter().map(|(id, _)| *id).collect();

        // Fetch all images for those review ids from the DB
        let rows = images::table
            .filter(images::review.eq_any(&review_ids))
            .load::<DbImage>(conn)
            .map_err(|e| {
                GqlApiError::internal("Error while loading images by review ID", e.to_string())
            })?;

        // Group images by their review id
        let mut review_map: HashMap<uuid::Uuid, Vec<DbImage>> = HashMap::new();
        for image in rows {
            review_map.entry(image.review).or_default().push(image);
        }

        // Construct a map of key => Vec<DbImage>
        for (review_id, key) in keys {
            let matched = review_map.get(review_id).cloned().unwrap_or_default();
            result_map.insert(key.clone(), matched);
        }

        Ok(())
    }

    fn load_by_occurrence_id(
        &self,
        keys: &[(uuid::Uuid, ImageLoaderKey)],
        conn: &mut diesel::PgConnection,
        result_map: &mut HashMap<ImageLoaderKey, Vec<DbImage>>,
    ) -> Result<(), async_graphql::Error> {
        // Nothing to do if there are no keys
        if keys.is_empty() {
            return Ok(());
        }

        // Collect the various occurrence ids
        let occ_ids: Vec<uuid::Uuid> = keys.iter().map(|(id, _)| *id).collect();

        // Fetch images with those occurrence ids from DB (via their review)
        let rows = images::table
            .inner_join(reviews::table)
            .filter(reviews::occurrence.eq_any(&occ_ids))
            .select((DbImage::as_select(), reviews::occurrence))
            .load::<(DbImage, uuid::Uuid)>(conn)
            .map_err(|e| {
                GqlApiError::internal("Error while loading images by occurrence ID", e.to_string())
            })?;

        // Group images by occurrence id
        let mut occ_map: HashMap<uuid::Uuid, Vec<DbImage>> = HashMap::new();
        for (image, occ_id) in rows {
            occ_map.entry(occ_id).or_default().push(image);
        }

        // Construct a map of key => Vec<DbImage>
        for (occ_id, key) in keys {
            let matched = occ_map.get(occ_id).cloned().unwrap_or_default();
            result_map.insert(key.clone(), matched);
        }

        Ok(())
    }

    fn load_by_dish_id(
        &self,
        keys: &[(uuid::Uuid, ImageLoaderKey)],
        conn: &mut diesel::PgConnection,
        result_map: &mut HashMap<ImageLoaderKey, Vec<DbImage>>,
    ) -> Result<(), async_graphql::Error> {
        // Nothing to do if there are no keys
        if keys.is_empty() {
            return Ok(());
        }

        // Collect the various dish ids
        let dish_ids: Vec<uuid::Uuid> = keys.iter().map(|(id, _)| *id).collect();

        // Fetch images with those dish ids from DB (via their review's occurrence)
        let rows = images::table
            .inner_join(reviews::table)
            .inner_join(occurrences::table)
            .filter(occurrences::dish.eq_any(&dish_ids))
            .select((DbImage::as_select(), occurrences::dish))
            .load::<(DbImage, uuid::Uuid)>(conn)
            .map_err(|e| {
                GqlApiError::internal("Error while loading images by dish ID", e.to_string())
            })?;

        // Group images by dish id
        let mut dish_map: HashMap<uuid::Uuid, Vec<DbImage>> = HashMap::new();
        for (image, dish_id) in rows {
            dish_map.entry(dish_id).or_default().push(image);
        }

        // Construct a map of key => Vec<DbImage>
        for (dish_id, key) in keys {
            let matched = dish_map.get(dish_id).cloned().unwrap_or_default();
            result_map.insert(key.clone(), matched);
        }

        Ok(())
    }
}

impl Loader<ImageLoaderKey> for ImageLoader {
    type Value = Vec<DbImage>;
    type Error = async_graphql::Error;

    async fn load(
        &self,
        keys: &[ImageLoaderKey],
    ) -> Result<HashMap<ImageLoaderKey, Self::Value>, Self::Error> {
        // Get DB connection
        let conn = &mut get_conn_from_pool(&self.pool)?;

        let mut result_map = HashMap::new();

        // Group keys by variant
        let mut by_image_id = vec![];
        let mut by_review_id = vec![];
        let mut by_occurrence_id = vec![];
        let mut by_dish_id = vec![];

        for key in keys {
            match key {
                ImageLoaderKey::ByImageId { id } => by_image_id.push((*id, key.clone())),
                ImageLoaderKey::ByReviewId { review_id } => {
                    by_review_id.push((*review_id, key.clone()))
                }
                ImageLoaderKey::ByOccurrenceId { occurrence_id } => {
                    by_occurrence_id.push((*occurrence_id, key.clone()))
                }
                ImageLoaderKey::ByDishId { dish_id } => by_dish_id.push((*dish_id, key.clone())),
            }
        }

        debug!(
            "Loading {} elements ({} img, {} rev, {} occ, {} dsh)",
            keys.len(),
            by_image_id.len(),
            by_review_id.len(),
            by_occurrence_id.len(),
            by_dish_id.len()
        );

        // NOTE: We execute one query per used variant. See the equivalent note in the review
        // loader for why these are kept as separate queries instead of a single joined one.
        self.load_by_image_id(&by_image_id, conn, &mut result_map)?;
        self.load_by_review_id(&by_review_id, conn, &mut result_map)?;
        self.load_by_occurrence_id(&by_occurrence_id, conn, &mut result_map)?;
        self.load_by_dish_id(&by_dish_id, conn, &mut result_map)?;

        Ok(result_map)
    }
}
