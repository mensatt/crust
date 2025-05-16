use async_graphql::dataloader::DataLoader;
use async_graphql::{Context, InputObject, Result, SimpleObject};
use diesel::prelude::*;
use log::debug;

use crate::db::models::dish::DbDish;
use crate::db::models::dish_alias::DbDishAlias;
use crate::graphql::error::GqlApiError;
use crate::graphql::queries::ReviewFilter;
use crate::graphql::util::get_conn_from_ctx;
use crate::schema::{dishes, dishes_aliases};
use crate::DishLoader;

use super::GqlReviewDataDish;

#[derive(Debug, SimpleObject)]
#[graphql(complex, name = "DishAlias")]
pub struct GqlDishAlias {
    pub alias_name: String,
    pub normalized_alias_name: String,

    // Internal fields, these should not be exposed via the API
    #[graphql(skip)]
    pub dish_id: uuid::Uuid,
}

// TODO: Extract to own file
#[async_graphql::ComplexObject]
impl GqlDishAlias {
    async fn dish(&self, ctx: &Context<'_>) -> Result<GqlDish> {
        debug!("Loading dish with ID '{}' within alias", self.dish_id);

        let loader = ctx.data::<DataLoader<DishLoader>>().map_err(|e| {
            GqlApiError::internal("Unable to get DishLoader from context", e.message)
        })?;

        let dish = loader
            .load_one(self.dish_id)
            .await
            .map_err(|e| {
                GqlApiError::internal(
                    format!(
                        "Unable to load dish with ID '{}' within alias via dish loader",
                        self.dish_id
                    ),
                    e.message,
                )
            })?
            .ok_or_else(|| {
                GqlApiError::not_found(format!("Dish with ID '{}' not found", self.dish_id))
            })?;
        Ok(dish.into())
    }
}

impl From<DbDishAlias> for GqlDishAlias {
    fn from(value: DbDishAlias) -> Self {
        GqlDishAlias {
            alias_name: value.alias_name,
            normalized_alias_name: value.normalized_alias_name,
            dish_id: value.dish,
        }
    }
}

#[derive(Debug, SimpleObject)]
#[graphql(complex, name = "Dish")]
pub struct GqlDish {
    pub id: uuid::Uuid,
    pub name_de: String,
    pub name_en: Option<String>,
}

impl From<DbDish> for GqlDish {
    fn from(value: DbDish) -> Self {
        GqlDish {
            id: value.id,
            name_de: value.name_de,
            name_en: value.name_en,
        }
    }
}

#[async_graphql::ComplexObject]
impl GqlDish {
    async fn review_data(
        &self,
        _ctx: &Context<'_>,
        filter: Option<ReviewFilter>,
    ) -> Result<GqlReviewDataDish> {
        // NOTE: Resolving fields for review_data is handled by the review and metadata
        //       resolvers of GqlReviewDataDish
        Ok(GqlReviewDataDish {
            dish_id: self.id,
            filter,
        })
    }

    async fn aliases(&self, ctx: &Context<'_>) -> Result<Vec<GqlDishAlias>> {
        // Get DB connection
        let conn = &mut get_conn_from_ctx(ctx)?;

        let results = dishes_aliases::table
            .filter(dishes_aliases::dish.eq(self.id))
            .select(DbDishAlias::as_select())
            .load(conn)
            .map_err(|e| {
                // NOTE: load will not return NotFound, so no need to match on error here
                GqlApiError::internal(
                    format!(
                        "Error while fetching dish aliases for dish with ID '{}'",
                        self.id
                    ),
                    e.to_string(),
                )
            })?;
        Ok(results.into_iter().map(Into::into).collect())
    }
}

#[derive(Debug, InputObject)]
pub struct DishFilter {
    pub dishes: Option<Vec<uuid::Uuid>>,
    pub name_de: Option<String>,
    pub name_en: Option<String>,
}

#[derive(Default)]
pub struct DishQueries;

#[async_graphql::Object]
impl DishQueries {
    async fn dishes(&self, ctx: &Context<'_>, filter: Option<DishFilter>) -> Result<Vec<GqlDish>> {
        // NOTE: Using the DishLoader is not beneficial here since (for now) we don't filter on ids

        // Get DB connection
        let conn = &mut get_conn_from_ctx(ctx)?;

        // Construct query
        let mut query = dishes::table.select(DbDish::as_select()).into_boxed();

        // Add necessary clauses depending on present filter values
        if let Some(f) = filter {
            if let Some(filter_dishes) = f.dishes {
                query = query.filter(dishes::id.eq_any(filter_dishes));
            }
            if let Some(filter_name_de) = f.name_de {
                query = query.filter(dishes::name_en.ilike(format!("%{}%", filter_name_de)));
            }
            if let Some(filter_name_en) = f.name_en {
                query = query.filter(dishes::name_en.ilike(format!("%{}%", filter_name_en)));
            }
        }

        // Return results
        let results: Vec<DbDish> = query
            .load(conn)
            .map_err(|e| GqlApiError::internal("Error while loading dishes", e.to_string()))?;
        Ok(results.into_iter().map(Into::into).collect())
    }
}
