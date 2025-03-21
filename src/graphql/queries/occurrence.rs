use async_graphql::{Context, Result, SimpleObject};
use diesel::prelude::*;

use crate::schema::{dishes, locations};
use crate::{
    db::{
        conn::DbPool,
        models::{dish::DbDish, location::DbLocation, occurrence::DbOccurrence},
    },
    graphql::util::{GqlDate, GqlTimestamp},
};

use super::{GqlDish, GqlLocation, GqlTag};

#[derive(Debug, SimpleObject)]
#[graphql(name = "Occurrence")]
pub struct GqlOccurrence {
    pub id: uuid::Uuid,
    pub location: GqlLocation,
    pub dish: GqlDish,
    pub side_dishes: Vec<GqlDish>,
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
    pub tags: Vec<GqlTag>,
    pub not_available_after: Option<GqlTimestamp>,
    // pub status: String, // NOTE: This is currently unused but present in the DB
    // TODO: Add review_data
}

impl From<(DbOccurrence, DbLocation, DbDish)> for GqlOccurrence {
    fn from(value: (DbOccurrence, DbLocation, DbDish)) -> Self {
        GqlOccurrence {
            id: value.0.id,
            location: value.1.into(),
            dish: value.2.into(),
            side_dishes: vec![], // TODO
            date: value.0.date.date_naive().into(),
            kj: value.0.kj,
            kcal: value.0.kcal,
            fat: value.0.fat,
            saturated_fat: value.0.saturated_fat,
            carbohydrates: value.0.carbohydrates,
            sugar: value.0.sugar,
            fiber: value.0.fiber,
            protein: value.0.protein,
            salt: value.0.salt,
            price_student: value.0.price_student,
            price_staff: value.0.price_staff,
            price_guest: value.0.price_guest,
            tags: vec![], // TODO
            not_available_after: value.0.not_available_after.map(Into::into),
        }
    }
}

#[derive(Default)]
pub struct OccurrenceQueries;

#[async_graphql::Object]
impl OccurrenceQueries {
    // TODO: Filter
    // TODO: Side-Dishes and Tags (n:m)
    async fn occurrences(&self, ctx: &Context<'_>) -> Result<Vec<GqlOccurrence>> {
        use crate::schema::occurrences::dsl::*;
        // Get DB connection
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        // Construct and execute query
        let query = occurrences
            .inner_join(locations::table)
            .inner_join(dishes::table)
            .select((
                DbOccurrence::as_select(),
                DbLocation::as_select(),
                DbDish::as_select(),
            ));

        let results = query
            .load::<(DbOccurrence, DbLocation, DbDish)>(conn)
            .expect("Error loading occurrences")
            .into_iter()
            .map(|tuple| tuple.into())
            .collect();

        Ok(results)
    }
}
