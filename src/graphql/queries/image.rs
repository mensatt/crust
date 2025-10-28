use async_graphql::dataloader::DataLoader;
use async_graphql::{Context, Result, SimpleObject};
use diesel::prelude::*;
use log::debug;

use crate::db::models::image::DbImage;
use crate::graphql::dataloaders::{ReviewLoader, ReviewLoaderKey};
use crate::graphql::error::GqlApiError;
use crate::graphql::util::get_conn_from_ctx;
use crate::schema::images::dsl::*;

use super::GqlReview;

#[derive(Debug, SimpleObject)]
#[graphql(complex, name = "Image")]
pub struct GqlImage {
    pub id: uuid::Uuid,

    // Internal fields which are not exposed via the API
    #[graphql(skip)]
    pub review_id: uuid::Uuid,
}

impl From<DbImage> for GqlImage {
    fn from(value: DbImage) -> Self {
        GqlImage {
            id: value.id,
            review_id: value.review,
        }
    }
}

#[async_graphql::ComplexObject]
impl GqlImage {
    async fn review(&self, ctx: &Context<'_>) -> Result<GqlReview> {
        debug!(
            "Loading review with ID '{}' for image with ID '{}'",
            self.review_id, self.id
        );

        let loader = ctx.data::<DataLoader<ReviewLoader>>().map_err(|e| {
            GqlApiError::internal("Unable to get ReviewLoader from context", e.message)
        })?;

        let rev = loader
            .load_one(ReviewLoaderKey::ByReviewId { id: self.review_id })
            .await
            .map_err(|e| {
                GqlApiError::internal(
                    format!(
                        "Unable to load review with ID '{}' within image via review loader",
                        self.review_id
                    ),
                    e.message,
                )
            })?
            .and_then(|v| v.into_iter().next())
            .ok_or_else(|| {
                GqlApiError::not_found(format!("Review with ID '{}' not found", self.review_id))
            })?;
        Ok(rev.into())
    }
}

#[derive(Default)]
pub struct ImageQueries;

#[async_graphql::Object]
impl ImageQueries {
    // TODO: Remove this query? It was not in the previous backend
    async fn images(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<GqlImage>> {
        // Get DB connection
        let conn = &mut get_conn_from_ctx(ctx)?;

        // Construct and execute query
        // TODO: If this query is kept, add a filter for approved images when not authenticated here
        let query = images.select(DbImage::as_select());
        let results = query
            .load(conn)
            .map_err(|e| GqlApiError::internal("Error while loading images", e.to_string()))?;
        Ok(results.into_iter().map(Into::into).collect())
    }
}
