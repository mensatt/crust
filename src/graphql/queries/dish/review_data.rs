use async_graphql::{dataloader::DataLoader, Context, Result, SimpleObject};
use bigdecimal::{BigDecimal, ToPrimitive};
use diesel::dsl::{avg, count};
use diesel::prelude::*;

use crate::db::conn::DbPool;
use crate::db::models::image::DbImage;
use crate::graphql::dataloaders::{ReviewLoader, ReviewLoaderKey};
use crate::graphql::queries::{GqlImage, GqlReview, ReviewFilter};
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
        // println!("Loading reviews for dish {}", self.dish_id);
        let loader = ctx.data::<DataLoader<ReviewLoader>>()?;
        let reviews = loader
            .load_one(ReviewLoaderKey::ByDishId {
                dish_id: self.dish_id,
                filter: self.filter,
            })
            .await?
            .unwrap_or_else(Vec::new);
        Ok(reviews.into_iter().map(Into::into).collect())
    }

    async fn metadata(&self, ctx: &Context<'_>) -> Result<GqlReviewMetadataDish> {
        // println!("Loading avg and count for dish: {}", self.dish_id);

        // Get DB connection
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

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
            .expect("Error loading dish review metadata");

        // Convert from big decimal to f32 (use 0.0 as fallback if conversion is not possible)
        // TODO: It would proably be best if the query returns an error here instead of silently
        //       defaulting to 0.0
        let average_stars = Some(raw_avg.and_then(|bigdec| bigdec.to_f32()).unwrap_or(0.0));

        Ok(GqlReviewMetadataDish {
            average_stars,
            review_count,
        })
    }

    async fn images(&self, ctx: &Context<'_>) -> Result<Vec<GqlImage>> {
        // Get DB connection
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        // Fetch images for this dish_id from database
        let results = reviews::table
            .inner_join(occurrences::table)
            .filter(occurrences::dish.eq(self.dish_id))
            .inner_join(images::table)
            .select(DbImage::as_select())
            .load::<DbImage>(conn)?;

        Ok(results.into_iter().map(Into::into).collect())
    }
}
