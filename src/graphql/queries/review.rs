use async_graphql::dataloader::DataLoader;
use async_graphql::{Context, InputObject, Result, SimpleObject};
use diesel::prelude::*;

use crate::db::models::image::DbImage;
use crate::db::{conn::DbPool, models::review::DbReview};
use crate::graphql::util::GqlTimestamp;
use crate::schema::images;
use crate::schema::reviews::dsl::*;
use crate::OccurrenceLoader;

use super::{GqlImage, GqlOccurrence};

#[derive(Debug, SimpleObject)]
#[graphql(complex, name = "Review")]
pub struct GqlReview {
    pub id: uuid::Uuid,
    pub display_name: Option<String>,
    pub stars: i64,
    pub text: Option<String>,
    pub created_at: GqlTimestamp,
    pub updated_at: GqlTimestamp,
    pub accepted_at: Option<GqlTimestamp>,

    // Internal fields which are not exposted via the API
    #[graphql(skip)]
    pub occurrence_id: uuid::Uuid,
}

#[async_graphql::ComplexObject]
impl GqlReview {
    async fn occurrence(&self, ctx: &Context<'_>) -> Result<GqlOccurrence> {
        // println!("Loading occurrence {}", self.occurrence_id);
        let loader = ctx.data::<DataLoader<OccurrenceLoader>>()?;
        let occ = loader
            .load_one(self.occurrence_id)
            .await?
            .ok_or("Occurrence not found")?;
        Ok(occ.into())
    }

    async fn images(&self, ctx: &Context<'_>) -> Result<Vec<GqlImage>> {
        // Get DB connection
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        let results = images::table
            .select(DbImage::as_select())
            .filter(images::review.eq(self.id))
            .load(conn)
            .expect("Error loading images for review");

        Ok(results.into_iter().map(Into::into).collect())
    }
}

impl From<DbReview> for GqlReview {
    fn from(value: DbReview) -> Self {
        GqlReview {
            id: value.id,
            display_name: value.display_name,
            stars: value.stars,
            text: value.text,
            created_at: value.created_at.into(),
            updated_at: value.updated_at.into(),
            accepted_at: value.accepted_at.map(Into::into),
            occurrence_id: value.occurrence,
        }
    }
}

#[derive(Debug, InputObject, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ReviewFilter {
    pub approved: Option<bool>,
}

#[derive(Default)]
pub struct ReviewQueries;

#[async_graphql::Object]
impl ReviewQueries {
    async fn reviews(
        &self,
        ctx: &Context<'_>,
        filter: Option<ReviewFilter>,
    ) -> async_graphql::Result<Vec<GqlReview>> {
        // NOTE: Using the ReviewLoader is not beneficial here since we don't exclusively filter on ids

        // Get DB connection
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        // Construct basic query
        let mut query = reviews.select(DbReview::as_select()).into_boxed();

        // Add neccessary clauses depending on present filter values
        if let Some(f) = filter {
            if let Some(filter_approved) = f.approved {
                if filter_approved {
                    query = query.filter(accepted_at.is_not_null());
                } else {
                    query = query.filter(accepted_at.is_null());
                }
            }
        }

        // Execute query
        let results = query.load(conn).expect("Error loading reviews");
        Ok(results.into_iter().map(Into::into).collect())
    }
}
