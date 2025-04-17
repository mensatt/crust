use async_graphql::dataloader::DataLoader;
use async_graphql::{Context, Result, SimpleObject};
use diesel::prelude::*;

use crate::graphql::dataloaders::{DishLoader, LocationLoader, SideDishLoader, TagLoader};
use crate::{
    db::{conn::DbPool, models::occurrence::DbOccurrence},
    graphql::{
        queries::{GqlDish, GqlLocation, GqlReviewDataOccurrence, GqlTag},
        util::{GqlDate, GqlTimestamp},
    },
};

#[derive(Debug, SimpleObject)]
#[graphql(complex, name = "Occurrence")]
pub struct GqlOccurrence {
    pub id: uuid::Uuid,
    pub date: GqlDate,
    pub kj: Option<i64>,
    pub kcal: Option<i64>,
    pub fat: Option<i64>,
    pub saturated_fat: Option<i64>,
    pub carbohydrates: Option<i64>,
    pub sugar: Option<i64>,
    pub fiber: Option<i64>,
    pub protein: Option<i64>,
    pub salt: Option<i64>,
    pub price_student: Option<i64>,
    pub price_staff: Option<i64>,
    pub price_guest: Option<i64>,
    pub not_available_after: Option<GqlTimestamp>,
    // pub status: String, // NOTE: This is currently unused but present in the DB

    // Internal fields, these should not be exposed via the API
    #[graphql(skip)]
    pub location_id: uuid::Uuid,
    #[graphql(skip)]
    pub dish_id: uuid::Uuid,
}

// Resolvers for nested fields
#[async_graphql::ComplexObject]
impl GqlOccurrence {
    async fn location(&self, ctx: &Context<'_>) -> Result<GqlLocation> {
        // println!("Loading location_id for {}", self.location_id);
        let loader = ctx.data::<DataLoader<LocationLoader>>()?;
        let loc = loader
            .load_one(self.location_id)
            .await?
            .ok_or("Location not found")?;
        Ok(loc.into())
    }

    async fn dish(&self, ctx: &Context<'_>) -> Result<GqlDish> {
        // println!("Loading dish for {}", self.dish_id);
        let loader = ctx.data::<DataLoader<DishLoader>>()?;
        let dish = loader
            .load_one(self.dish_id)
            .await?
            .ok_or("Dish not found")?;
        Ok(dish.into())
    }

    async fn tags(&self, ctx: &Context<'_>) -> Result<Vec<GqlTag>> {
        // println!("Loading tags for {}", self.id);
        let loader = ctx.data::<DataLoader<TagLoader>>()?;
        let tags = loader.load_one(self.id).await?.unwrap_or_else(Vec::new);
        Ok(tags.into_iter().map(Into::into).collect())
    }

    async fn side_dishes(&self, ctx: &Context<'_>) -> Result<Vec<GqlDish>> {
        // println!("Loading side dishes for {}", self.id);
        let loader = ctx.data::<DataLoader<SideDishLoader>>()?;
        let side_dishes = loader.load_one(self.id).await?.unwrap_or_else(Vec::new);
        Ok(side_dishes.into_iter().map(Into::into).collect())
    }

    async fn review_data(&self, _ctx: &Context<'_>) -> Result<GqlReviewDataOccurrence> {
        // NOTE: Resolving fields for review_data is handled by the review and metadata
        //       resolvers of GqlReviewDataOccurrence
        Ok(GqlReviewDataOccurrence {
            occurrence_id: self.id,
        })
    }
}

impl From<DbOccurrence> for GqlOccurrence {
    fn from(occ: DbOccurrence) -> Self {
        GqlOccurrence {
            id: occ.id,
            date: occ.date.date_naive().into(),
            kj: occ.kj,
            kcal: occ.kcal,
            fat: occ.fat,
            saturated_fat: occ.saturated_fat,
            carbohydrates: occ.carbohydrates,
            sugar: occ.sugar,
            fiber: occ.fiber,
            protein: occ.protein,
            salt: occ.salt,
            price_student: occ.price_student,
            price_staff: occ.price_staff,
            price_guest: occ.price_guest,
            not_available_after: occ.not_available_after.map(Into::into),
            // Internal fields
            location_id: occ.location,
            dish_id: occ.dish,
        }
    }
}

#[derive(Default)]
pub struct OccurrenceQueries;

#[async_graphql::Object]
impl OccurrenceQueries {
    // TODO: Filter
    async fn occurrences(&self, ctx: &Context<'_>) -> Result<Vec<GqlOccurrence>> {
        use crate::schema::occurrences::dsl::*;
        // Get DB connection
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        // Fetch required occurrences
        let data = occurrences
            .select(DbOccurrence::as_select())
            .load(conn)
            .expect("Error loading occurrences");

        // Convert from DbOccurrence to GqlOccurrence
        Ok(data.into_iter().map(Into::into).collect())
    }
}
