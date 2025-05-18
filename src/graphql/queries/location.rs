use async_graphql::{Context, InputObject, SimpleObject};
use diesel::prelude::*;

use crate::db::models::location::DbLocation;
use crate::graphql::error::GqlApiError;
use crate::graphql::util::get_conn_from_ctx;

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

#[derive(Debug, InputObject)]
pub struct LocationFilter {
    pub ids: Option<Vec<uuid::Uuid>>,
    pub external_ids: Option<Vec<i64>>,
    pub names: Option<Vec<String>>,
    pub visible: Option<bool>,
}

#[derive(Default)]
pub struct LocationQueries;

#[async_graphql::Object]
impl LocationQueries {
    async fn locations(
        &self,
        ctx: &Context<'_>,
        filter: Option<LocationFilter>,
    ) -> async_graphql::Result<Vec<GqlLocation>> {
        use crate::schema::locations::dsl::*;
        // NOTE: Using the LocationLoader is not beneficial here since we don't exclusively filter on ids

        // Get DB connection
        let conn = &mut get_conn_from_ctx(ctx)?;

        // Construct query
        let mut query = locations.select(DbLocation::as_select()).into_boxed();

        // Add necessary clauses depending on present filter values
        if let Some(f) = filter {
            if let Some(filter_ids) = f.ids {
                query = query.filter(id.eq_any(filter_ids));
            }
            if let Some(filter_external_ids) = f.external_ids {
                query = query.filter(external_id.eq_any(filter_external_ids));
            }
            if let Some(filter_names) = f.names {
                query = query.filter(name.eq_any(filter_names));
            }
            if let Some(filter_visible) = f.visible {
                query = query.filter(visible.eq(filter_visible));
            }
        }

        // Execute query and return results
        let results = query
            .load(conn)
            .map_err(|e| GqlApiError::internal("Error while loading locations", e.to_string()))?;

        Ok(results.into_iter().map(Into::into).collect())
    }
}
