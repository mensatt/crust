use async_graphql::{Context, SimpleObject};
use diesel::prelude::*;

use crate::db::{conn::DbPool, models::location::DbLocation};

#[derive(Debug, SimpleObject)]
#[graphql(name = "Location")]
pub struct GqlLocation {
    pub id: uuid::Uuid,
    pub external_id: i64,
    pub name: String,
    pub visible: bool,
}

impl From<DbLocation> for GqlLocation {
    fn from(value: DbLocation) -> Self {
        GqlLocation {
            id: value.id,
            external_id: value.external_id,
            name: value.name,
            visible: value.visible,
        }
    }
}

#[derive(Default)]
pub struct LocationQueries;

#[async_graphql::Object]
impl LocationQueries {
    // TODO: Filter
    async fn locations(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<GqlLocation>> {
        use crate::schema::locations::dsl::*;

        // Get DB conn
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        // Construct and execute query
        let query = locations.select(DbLocation::as_select());
        let results = query.load(conn).expect("Error loading locations");
        Ok(results.into_iter().map(Into::into).collect())
    }
}
