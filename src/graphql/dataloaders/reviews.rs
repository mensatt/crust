use std::collections::HashMap;

use async_graphql::dataloader::Loader;
use diesel::prelude::*;
use log::debug;

use crate::db::{conn::DbPool, models::review::DbReview};
use crate::graphql::error::GqlApiError;
use crate::graphql::queries::ReviewFilter;
use crate::graphql::util::get_conn_from_pool;
use crate::schema::{occurrences, reviews};

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum ReviewLoaderKey {
    ByReviewId {
        id: uuid::Uuid,
    },
    ByOccurrenceId {
        occurrence_id: uuid::Uuid,
        filter: Option<ReviewFilter>,
    },
    ByDishId {
        dish_id: uuid::Uuid,
        filter: Option<ReviewFilter>,
    },
}

pub struct ReviewLoader {
    pub pool: DbPool,
}

impl ReviewLoader {
    fn matches_filter(review: &DbReview, filter: Option<ReviewFilter>) -> bool {
        match filter {
            // Some(approved: Some(true)) == "filter present; only keep approved reviews"
            Some(ReviewFilter {
                approved: Some(true),
            }) => review.accepted_at.is_some(),
            // Some(approved: Some(false)) == "filter present; only keep unapproved reviews"
            Some(ReviewFilter {
                approved: Some(false),
            }) => review.accepted_at.is_none(),
            // Filter present, but no preference: Keep every review
            Some(ReviewFilter { approved: None }) => true,
            // No filter present: Keep every review
            None => true,
        }
    }

    fn load_by_review_id(
        &self,
        keys: &[(uuid::Uuid, ReviewLoaderKey)],
        conn: &mut diesel::PgConnection,
        result_map: &mut HashMap<ReviewLoaderKey, Vec<DbReview>>,
    ) -> Result<(), async_graphql::Error> {
        // Nothing to do if there are no keys
        if keys.is_empty() {
            return Ok(());
        }

        // Collect the various review ids
        let ids: Vec<uuid::Uuid> = keys.iter().map(|(id, _)| *id).collect();

        // Fetch all those ids from the DB
        let reviews = reviews::table
            .filter(reviews::id.eq_any(&ids))
            .load::<DbReview>(conn)
            .map_err(|e| {
                GqlApiError::internal("Error while loading reviews by ID", e.to_string())
            })?;

        // Group reviews by review_id
        let review_map: HashMap<uuid::Uuid, DbReview> =
            reviews.into_iter().map(|r| (r.id, r)).collect();

        // Construct a map of key => reviews
        for (id, key) in keys {
            let matched = review_map.get(id).cloned().into_iter().collect();
            result_map.insert(key.clone(), matched);
        }

        Ok(())
    }

    fn load_by_occurrence_id(
        &self,
        keys: &[(uuid::Uuid, Option<ReviewFilter>, ReviewLoaderKey)],
        conn: &mut diesel::PgConnection,
        result_map: &mut HashMap<ReviewLoaderKey, Vec<DbReview>>,
    ) -> Result<(), async_graphql::Error> {
        // Nothing to do if there are no keys
        if keys.is_empty() {
            return Ok(());
        }

        // Collect the various occurrence ids
        let occ_ids: Vec<_> = keys.iter().map(|(id, _, _)| *id).collect();

        // Fetch reviews with those occurrence ids from DB
        let reviews = reviews::table
            .filter(reviews::occurrence.eq_any(&occ_ids))
            .load::<DbReview>(conn)
            .map_err(|e| {
                GqlApiError::internal(
                    "Error while loading reviews by occurrence ID",
                    e.to_string(),
                )
            })?;

        // Group reviews by occurrence_id
        let mut occ_map: HashMap<uuid::Uuid, Vec<DbReview>> = HashMap::new();
        for review in reviews {
            occ_map.entry(review.occurrence).or_default().push(review);
        }

        // Construct a map of key => Vec<DbReview> by using the key's filter
        for (occ_id, filter, key) in keys {
            let matched = occ_map
                // Get all reviews for this occurrence id
                .get(occ_id)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                // Only keep the ones that match the key's filter
                .filter(|r| Self::matches_filter(r, *filter))
                .collect();
            result_map.insert(key.clone(), matched);
        }

        Ok(())
    }

    fn load_by_dish_id(
        &self,
        keys: &[(uuid::Uuid, Option<ReviewFilter>, ReviewLoaderKey)],
        conn: &mut diesel::PgConnection,
        result_map: &mut HashMap<ReviewLoaderKey, Vec<DbReview>>,
    ) -> Result<(), async_graphql::Error> {
        // Nothing to do if there are no keys
        if keys.is_empty() {
            return Ok(());
        }

        // Collect the various dish ids
        let dish_ids: Vec<_> = keys.iter().map(|(id, _, _)| *id).collect();

        // Fetch reviews with those dish ids from DB
        let reviews = reviews::table
            .inner_join(occurrences::table)
            .filter(occurrences::dish.eq_any(&dish_ids))
            .select((DbReview::as_select(), occurrences::dish))
            .load::<(DbReview, uuid::Uuid)>(conn)
            .map_err(|e| {
                GqlApiError::internal("Error while loading reviews by dish ID", e.to_string())
            })?;

        // Group reviews by dish ID
        let mut dish_map: HashMap<uuid::Uuid, Vec<DbReview>> = HashMap::new();
        for (review, dish_id) in reviews {
            dish_map.entry(dish_id).or_default().push(review);
        }

        // Construct a map of key => Vec<DbReview> by using the key's filter
        for (dish_id, filter, key) in keys {
            let matched = dish_map
                // Get all reviews for this dish id
                .get(dish_id)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                // Only keep the ones that match the key's filter
                .filter(|r| Self::matches_filter(r, *filter))
                .collect();
            result_map.insert(key.clone(), matched);
        }

        Ok(())
    }
}

impl Loader<ReviewLoaderKey> for ReviewLoader {
    type Value = Vec<DbReview>;
    type Error = async_graphql::Error;

    async fn load(
        &self,
        keys: &[ReviewLoaderKey],
    ) -> Result<HashMap<ReviewLoaderKey, Self::Value>, Self::Error> {
        // Get DB connection
        let conn = &mut get_conn_from_pool(&self.pool)?;

        let mut result_map = HashMap::new();

        // Group keys by variant
        let mut by_review_id = vec![];
        let mut by_occurrence_id = vec![];
        let mut by_dish_id = vec![];

        for key in keys {
            match key {
                ReviewLoaderKey::ByReviewId { id } => by_review_id.push((*id, key.clone())),
                ReviewLoaderKey::ByOccurrenceId {
                    occurrence_id,
                    filter,
                } => by_occurrence_id.push((*occurrence_id, *filter, key.clone())),
                ReviewLoaderKey::ByDishId { dish_id, filter } => {
                    by_dish_id.push((*dish_id, *filter, key.clone()))
                }
            }
        }

        debug!(
            "Loading {} elements ({} rev, {} occ, {} dsh)",
            keys.len(),
            by_review_id.len(),
            by_occurrence_id.len(),
            by_dish_id.len()
        );

        // NOTE: We execute three separate queries here even though one review might be present in
        // multiple of them (which is inefficient).
        // It was decided to do it this way because constructing a single large query is
        //  a) overly complex (especially when filters come into play)
        //  b) the observed benefit was/is small because of the way async_graphql dataloader
        //     scheduling works, which leads to frequent requests with only one or two ids here.
        self.load_by_review_id(&by_review_id, conn, &mut result_map)?;
        self.load_by_occurrence_id(&by_occurrence_id, conn, &mut result_map)?;
        self.load_by_dish_id(&by_dish_id, conn, &mut result_map)?;

        Ok(result_map)
    }
}
