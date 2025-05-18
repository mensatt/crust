use async_graphql::{dataloader::DataLoader, Context, Result, SimpleObject};
use bigdecimal::{BigDecimal, ToPrimitive};
use diesel::dsl::{avg, count};
use diesel::prelude::*;
use diesel::result::Error::NotFound;
use log::debug;

use crate::db::models::image::DbImage;
use crate::graphql::dataloaders::{ReviewLoader, ReviewLoaderKey};
use crate::graphql::error::GqlApiError;
use crate::graphql::queries::{GqlImage, GqlReview, ReviewFilter};
use crate::graphql::util::get_conn_from_ctx;
use crate::schema::{images, occurrences, reviews};

#[derive(Debug, SimpleObject)]
#[graphql(name = "ReviewMetadataDish")]
pub struct GqlReviewMetadataDish {
    average_stars: Option<f32>,
    review_count: i64,
}

pub struct GqlReviewDataDish {
    pub dish_id: uuid::Uuid,
    pub filter: Option<ReviewFilter>,
}

#[async_graphql::Object(name = "ReviewDataDish")]
impl GqlReviewDataDish {
    async fn reviews(&self, ctx: &Context<'_>) -> Result<Vec<GqlReview>> {
        debug!("Loading reviews for dish with ID '{}'", self.dish_id);
        let loader = ctx.data::<DataLoader<ReviewLoader>>().map_err(|e| {
            GqlApiError::internal("Unable to get ReviewLoader from context", e.message)
        })?;
        let reviews = loader
            .load_one(ReviewLoaderKey::ByDishId {
                dish_id: self.dish_id,
                filter: self.filter,
            })
            .await
            .map_err(|e| {
                GqlApiError::internal(
                    format!(
                        "Unable to load reviews for dish with ID '{}' via review loader",
                        self.dish_id
                    ),
                    e.message,
                )
            })?
            .unwrap_or_else(Vec::new);
        Ok(reviews.into_iter().map(Into::into).collect())
    }

    async fn metadata(&self, ctx: &Context<'_>) -> Result<GqlReviewMetadataDish> {
        debug!("Loading avg and count for dish with ID '{}'", self.dish_id);

        // Get DB connection
        let conn = &mut get_conn_from_ctx(ctx)?;

        // NOTE: PostgreSQL's avg() function returns NUMERIC (a fixed-point decimal)
        //       for averages over integer columns (which stars are).
        //       Since that number cannot be converted into an float without losses
        //       we need to use the bigdecimal crate here and convert to f32 later manually
        //       Also see: https://github.com/diesel-rs/diesel/issues/841
        let (raw_avg, review_count) = reviews::table
            .inner_join(occurrences::table)
            .filter(occurrences::dish.eq(self.dish_id))
            .select((avg(reviews::stars), count(reviews::id)))
            .first::<(Option<BigDecimal>, i64)>(conn)
            .map_err(|e| match e {
                NotFound => GqlApiError::not_found(format!(
                    "Reviews for dish with ID '{}' not found",
                    self.dish_id
                )),
                _ => GqlApiError::internal(
                    format!(
                        "Error while loading review metadata for dish with ID '{}'",
                        self.dish_id
                    ),
                    e.to_string(),
                ),
            })?;

        // If present, convert from BigDecimal to f32 (and return error if conversion fails)
        // NOTE: This will be None (null after conversion) instead of 0 if raw_avg was None.
        //       This behaviour differs from the old backend implementation, but is compliant
        //       with it's GraphQL schema.
        let average_stars = raw_avg
            .as_ref()
            .map(|bigdec| {
                bigdec.to_f32().ok_or_else(|| {
                    GqlApiError::internal(
                        format!("Unable to convert postgres average '{:?}' to f32", raw_avg),
                        "",
                    )
                })
            })
            .transpose()?;

        Ok(GqlReviewMetadataDish {
            average_stars,
            review_count,
        })
    }

    async fn images(&self, ctx: &Context<'_>) -> Result<Vec<GqlImage>> {
        // Get DB connection
        let conn = &mut get_conn_from_ctx(ctx)?;

        // Fetch images for this dish_id from database
        let results = reviews::table
            .inner_join(occurrences::table)
            .filter(occurrences::dish.eq(self.dish_id))
            .inner_join(images::table)
            .select(DbImage::as_select())
            .load::<DbImage>(conn)
            .map_err(|e| {
                GqlApiError::internal(
                    format!(
                        "Error while loading review images of dish with ID '{}'",
                        self.dish_id
                    ),
                    e.to_string(),
                )
            })?;

        Ok(results.into_iter().map(Into::into).collect())
    }
}
