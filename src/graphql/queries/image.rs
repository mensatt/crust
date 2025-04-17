use async_graphql::dataloader::DataLoader;
use async_graphql::{Context, Result, SimpleObject};
use diesel::prelude::*;

use crate::db::{conn::DbPool, models::image::DbImage};
use crate::graphql::dataloaders::ReviewLoader;
use crate::schema::images::dsl::*;

use super::GqlReview;

#[derive(Debug, SimpleObject)]
#[graphql(complex, name = "Image")]
pub struct GqlImage {
    pub id: uuid::Uuid,

    // Internal fields which are not exposted via the API
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
        // println!("Loading review for image_id {}", self.id);
        let loader = ctx.data::<DataLoader<ReviewLoader>>()?;
        let rev = loader
            .load_one(self.review_id)
            .await?
            .ok_or("Location not found")?;
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
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        // Construct and execute query
        // TODO: If this query is kept, add a filter for approved images when not authenticated here
        let query = images.select(DbImage::as_select());
        let results = query.load(conn).expect("Error loading images");
        Ok(results.into_iter().map(Into::into).collect())
    }
}
