use async_graphql::{Context, InputObject, Result, SimpleObject};
use diesel::prelude::*;

use crate::db::{conn::DbPool, models::dish::DbDish};
use crate::schema::dishes;

use super::GqlReviewDataDish;

#[derive(Debug, SimpleObject)]
#[graphql(complex, name = "Dish")]
pub struct GqlDish {
    pub id: uuid::Uuid,
    pub name_de: String,
    pub name_en: Option<String>,
    // TODO: aliases
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
    async fn review_data(&self, _ctx: &Context<'_>) -> Result<GqlReviewDataDish> {
        // NOTE: Resolving fields for review_data is handled by the review and metadata
        //       resolvers of GqlReviewDataDish
        Ok(GqlReviewDataDish { dish_id: self.id })
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

        // Get DB conn
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        // Construct query
        let mut query = dishes::table.select(DbDish::as_select()).into_boxed();

        // Add neccessary clauses depending on present filter values
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
        let results: Vec<DbDish> = query.load(conn).expect("Error loading dishes");
        Ok(results.into_iter().map(Into::into).collect())
    }
}
