use async_graphql::{Context, Result, SimpleObject};
use diesel::prelude::*;

use crate::db::models::tag::DbTag;
use crate::schema::{dishes, locations, occurrences_tags, tags};
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

// Placeholder for mutations (for now)
impl From<(DbOccurrence, DbLocation, DbDish)> for GqlOccurrence {
    fn from(tuple: (DbOccurrence, DbLocation, DbDish)) -> Self {
        let (occ, loc, dish) = tuple;
        (occ, loc, dish, Vec::new()).into()
    }
}

impl From<(DbOccurrence, DbLocation, DbDish, Vec<DbTag>)> for GqlOccurrence {
    fn from((occ, loc, dish, tags): (DbOccurrence, DbLocation, DbDish, Vec<DbTag>)) -> Self {
        GqlOccurrence {
            id: occ.id,
            location: loc.into(),
            dish: dish.into(),
            side_dishes: vec![], // TODO
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
            tags: tags.into_iter().map(Into::into).collect(),
            not_available_after: occ.not_available_after.map(Into::into),
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

        // Step 1: Get main occurrence with associated location and dish data
        let query = occurrences
            .inner_join(locations::table)
            .inner_join(dishes::table)
            .select((
                DbOccurrence::as_select(),
                DbLocation::as_select(),
                DbDish::as_select(),
            ));

        let base_data = query
            .load::<(DbOccurrence, DbLocation, DbDish)>(conn)
            .expect("Error loading occurrences");

        // TODO: This works (and is reasonably efficient if query contains tags) but GQL resolvers
        // are the better approach.
        // NOTE: To avoid N+1 problem use dataloader as described here:
        // https://async-graphql.github.io/async-graphql/en/dataloader.html

        // Step 2: Collect occurrence ids...
        let occurrence_ids: Vec<uuid::Uuid> = base_data.iter().map(|(occ, _, _)| occ.id).collect();
        // ... and fetch all their tag data
        let tag_joins = occurrences_tags::table
            .filter(occurrences_tags::occurrence.eq_any(&occurrence_ids))
            .inner_join(tags::table)
            .select((occurrences_tags::occurrence, DbTag::as_select()))
            .load::<(uuid::Uuid, DbTag)>(conn)
            .expect("Error loading occurrence tags");

        // Step 3: Group the tags together by their occurrence ids
        use std::collections::HashMap;
        let mut tags_map: HashMap<uuid::Uuid, Vec<DbTag>> = HashMap::new();
        for (occurrence_id, tag) in tag_joins {
            tags_map.entry(occurrence_id).or_default().push(tag);
        }

        // Step 4: Map to GqlOccurrence with occurrence, location and tag data
        let gql_occurrences = base_data
            .into_iter()
            .map(|(db_occurrence, db_location, db_dish)| {
                let tag_vector = tags_map.remove(&db_occurrence.id).unwrap_or_default();
                (db_occurrence, db_location, db_dish, tag_vector).into()
            })
            .collect();

        Ok(gql_occurrences)
    }
}
