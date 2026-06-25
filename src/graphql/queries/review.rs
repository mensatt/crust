use async_graphql::dataloader::DataLoader;
use async_graphql::{Context, InputObject, Result, SimpleObject};
use diesel::prelude::*;
use log::debug;

use crate::db::models::review::DbReview;
use crate::graphql::error::GqlApiError;
use crate::graphql::util::{get_conn_from_ctx, GqlTimestamp};
use crate::schema::reviews::dsl::*;
use crate::{ImageLoader, OccurrenceLoader};

use super::{GqlImage, GqlOccurrence};

#[derive(Debug, Clone, SimpleObject)]
#[graphql(complex, name = "Review")]
pub struct GqlReview {
    pub id: uuid::Uuid,
    pub display_name: Option<String>,
    pub stars: i64,
    pub text: Option<String>,
    pub created_at: GqlTimestamp,
    pub updated_at: GqlTimestamp,
    pub accepted_at: Option<GqlTimestamp>,

    // Internal fields which are not exposed via the API
    #[graphql(skip)]
    pub occurrence_id: uuid::Uuid,
}

#[async_graphql::ComplexObject]
impl GqlReview {
    async fn occurrence(&self, ctx: &Context<'_>) -> Result<GqlOccurrence> {
        debug!(
            "Loading occurrence with ID '{}' for review with ID '{}'",
            self.occurrence_id, self.id
        );
        let loader = ctx.data::<DataLoader<OccurrenceLoader>>().map_err(|e| {
            GqlApiError::internal("Unable to get OccurrenceLoader from context", e.message)
        })?;
        let occ = loader
            .load_one(self.occurrence_id)
            .await
            .map_err(|e| {
                GqlApiError::internal(
                    format!(
                        "Unable to load occurrence with ID '{}' for review with ID '{}' via occurrence loader",
                        self.occurrence_id, self.id
                    ),
                    e.message,
                )
            })?
            .ok_or_else(|| {
                GqlApiError::not_found(format!("Occurrence with ID '{}' not found", self.occurrence_id))
            })?;
        Ok(occ.into())
    }

    pub async fn images(&self, ctx: &Context<'_>) -> Result<Vec<GqlImage>> {
        let loader = ctx.data::<DataLoader<ImageLoader>>().map_err(|e| {
            GqlApiError::internal("Unable to get ImageLoader from context", e.message)
        })?;

        let results = loader
            .load_one(self.id)
            .await
            .map_err(|e| {
                GqlApiError::internal(
                    format!(
                        "Error while loading images for review with ID '{}' via image loader",
                        self.id
                    ),
                    e.message,
                )
            })?
            .unwrap_or_default();

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
        let conn = &mut get_conn_from_ctx(ctx)?;

        // Construct basic query
        let mut query = reviews.select(DbReview::as_select()).into_boxed();

        // Add necessary clauses depending on present filter values
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
        let results = query
            .load(conn)
            .map_err(|e| GqlApiError::internal("Error while loading reviews", e.to_string()))?;

        Ok(results.into_iter().map(Into::into).collect())
    }
}
