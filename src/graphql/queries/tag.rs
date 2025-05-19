use async_graphql::{Context, Result, SimpleObject};
use diesel::prelude::*;

use crate::db::models::tag::{DbTag, TagPriority};
use crate::graphql::error::GqlApiError;
use crate::graphql::util::get_conn_from_ctx;
use crate::schema::tags::dsl::*;

#[derive(Debug, SimpleObject)]
#[graphql(name = "Tag")]
pub struct GqlTag {
    pub key: String,
    pub name: String,
    pub description: String,
    pub short_name: Option<String>,
    pub priority: TagPriority,
    pub is_allergy: bool,
}

impl From<DbTag> for GqlTag {
    fn from(value: DbTag) -> Self {
        GqlTag {
            key: value.key,
            name: value.name,
            description: value.description,
            short_name: value.short_name,
            priority: value.priority,
            is_allergy: value.is_allergy,
        }
    }
}

#[derive(Default)]
pub struct TagQueries;

#[async_graphql::Object]
impl TagQueries {
    async fn tags(&self, ctx: &Context<'_>) -> Result<Vec<GqlTag>> {
        // NOTE: Using the TagLoader is not beneficial here since (for now) we don't filter on ids

        // Get DB connection
        let conn = &mut get_conn_from_ctx(ctx)?;

        // Construct and execute query
        let results = tags
            .select(DbTag::as_select())
            .load(conn)
            .map_err(|e| GqlApiError::internal("Error while loading tags", e.to_string()))?;

        Ok(results.into_iter().map(Into::into).collect())
    }
}
