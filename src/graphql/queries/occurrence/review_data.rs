use async_graphql::{dataloader::DataLoader, Context, Result, SimpleObject};
use bigdecimal::{BigDecimal, ToPrimitive};
use diesel::dsl::{avg, count};
use diesel::prelude::*;

use crate::db::conn::DbPool;
use crate::db::models::image::DbImage;
use crate::graphql::queries::{GqlImage, GqlReview};
use crate::schema::{images, reviews};
use crate::OccurrenceReviewLoader;

#[derive(Debug, SimpleObject)]
#[graphql(name = "ReviewMetadataOccurrence")]
pub struct GqlReviewMetadataOccurrence {
    average_stars: Option<f32>,
    review_count: i64,
}

pub struct GqlReviewDataOccurrence {
    pub occurrence_id: uuid::Uuid,
}

#[async_graphql::Object(name = "ReviewDataOccurrence")]
impl GqlReviewDataOccurrence {
    async fn reviews(&self, ctx: &Context<'_>) -> Result<Vec<GqlReview>> {
        // println!("Loading reviews for occurrence {}", self.occurrence_id);
        let loader = ctx.data::<DataLoader<OccurrenceReviewLoader>>()?;
        let reviews = loader
            .load_one(self.occurrence_id)
            .await?
            .unwrap_or_else(Vec::new);
        Ok(reviews.into_iter().map(Into::into).collect())
    }

    async fn metadata(&self, ctx: &Context<'_>) -> Result<GqlReviewMetadataOccurrence> {
        // println!("Loading avg and count for occurrence: {}", self.occurrence_id);

        // Get DB connection
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        // NOTE: PostgreSQL's avg() function returns NUMERIC (a fixed-point decimal)
        //       for averages over integer columns (which stars are).
        //       Since that number cannot be converted into an float without losses
        //       we need to use the bigdecimal crate here and convert to f32 later manually
        //       Also see: https://github.com/diesel-rs/diesel/issues/841
        let (raw_avg, review_count) = reviews::table
            .filter(reviews::occurrence.eq(self.occurrence_id))
            .select((avg(reviews::stars), count(reviews::id)))
            .first::<(Option<BigDecimal>, i64)>(conn)
            .expect("Error loading occurrence review metadata");

        // Convert from big decimal to f32 (use 0.0 as fallback if conversion is not possible)
        // TODO: It would proably be best if the query returns an error here instead of silently
        //       defaulting to 0.0
        let average_stars = Some(raw_avg.and_then(|bigdec| bigdec.to_f32()).unwrap_or(0.0));

        Ok(GqlReviewMetadataOccurrence {
            average_stars,
            review_count,
        })
    }

    async fn images(&self, ctx: &Context<'_>) -> Result<Vec<GqlImage>> {
        // Get DB connection
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        // Fetch images for this occurrence_id from database
        let results = reviews::table
            .filter(reviews::occurrence.eq(self.occurrence_id))
            .inner_join(images::table)
            .select(DbImage::as_select())
            .load::<DbImage>(conn)?;

        Ok(results.into_iter().map(Into::into).collect())
    }
}
