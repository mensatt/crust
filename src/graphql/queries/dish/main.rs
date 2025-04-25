use async_graphql::dataloader::DataLoader;
use async_graphql::{Context, InputObject, Result, SimpleObject};
use diesel::prelude::*;

use crate::db::models::dish_alias::DbDishAlias;
use crate::db::{conn::DbPool, models::dish::DbDish};
use crate::graphql::queries::ReviewFilter;
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

#[async_graphql::ComplexObject]
impl GqlDishAlias {
    async fn dish(&self, ctx: &Context<'_>) -> Result<GqlDish> {
        // println!("Loading dish for {}", self.dish_id);
        let loader = ctx.data::<DataLoader<DishLoader>>()?;
        let dish = loader
            .load_one(self.dish_id)
            .await?
            .ok_or("Dish not found")?;
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
    async fn review_data(&self, _ctx: &Context<'_>, filter: Option<ReviewFilter>) -> Result<GqlReviewDataDish> {
        // NOTE: Resolving fields for review_data is handled by the review and metadata
        //       resolvers of GqlReviewDataDish
        Ok(GqlReviewDataDish { dish_id: self.id, filter})
    }

    async fn aliases(&self, ctx: &Context<'_>) -> Result<Vec<GqlDishAlias>> {
        // Get DB connection
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        let results = dishes_aliases::table
            .filter(dishes_aliases::dish.eq(self.id))
            .select(DbDishAlias::as_select())
            .load(conn)
            .expect("Error loading dish aliases");
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
