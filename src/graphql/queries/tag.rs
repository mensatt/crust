use async_graphql::{Context, Result, SimpleObject};
use diesel::prelude::*;

use crate::db::{
    conn::DbPool,
    models::tag::{DbTag, TagPriority},
};

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
        use crate::schema::tags::dsl::*;

        // Get DB conn
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        // Construct and execute query
        let query = tags.select(DbTag::as_select());
        let results: Vec<DbTag> = query.load(conn).expect("Error loading tags");
        Ok(results.into_iter().map(Into::into).collect())
    }
}
