use async_graphql::{Context, InputObject, Result};
use diesel::prelude::*;

use crate::db::{
    conn::DbPool,
    models::occurrence::{DbOccurrence, DbOccurrenceChangeset},
};
use crate::graphql::{queries::GqlOccurrence, util::GqlDate};
use crate::schema::occurrences;

#[derive(Debug, InputObject)]
pub struct CreateOccurrenceInput {
    pub date: Option<GqlDate>,
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
    // Foreign keys
    pub dish: uuid::Uuid,
    pub location: uuid::Uuid,
    pub side_dishes: Option<Vec<uuid::Uuid>>, // TODO: Currently unused
    pub tags: Option<Vec<String>>,            // TODO: Currently unused
}

#[derive(Debug, InputObject, Clone, Copy)]
pub struct UpdateOccurrenceInput {
    pub id: uuid::Uuid,
    pub date: Option<GqlDate>,
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
    pub dish: Option<uuid::Uuid>,
}

impl From<UpdateOccurrenceInput> for DbOccurrenceChangeset {
    fn from(value: UpdateOccurrenceInput) -> Self {
        DbOccurrenceChangeset {
            dish: value.dish,
            date: value.date.map(Into::into),
            kj: value.kj,
            kcal: value.kcal,
            fat: value.fat,
            saturated_fat: value.saturated_fat,
            carbohydrates: value.carbohydrates,
            sugar: value.sugar,
            fiber: value.fiber,
            protein: value.protein,
            salt: value.salt,
            price_student: value.price_student,
            price_staff: value.price_student,
            price_guest: value.price_guest,
            // Optional fields, that are unused by GraphQL
            location: None,
            not_available_after: None,
            status: None,
        }
    }
}

#[derive(Default)]
pub struct OccurrenceMutations;

#[async_graphql::Object]
impl OccurrenceMutations {
    async fn create_occurrence(
        &self,
        ctx: &Context<'_>,
        input: CreateOccurrenceInput,
    ) -> Result<GqlOccurrence> {
        // Get DB connection
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        let new_occurrence = DbOccurrence {
            id: uuid::Uuid::new_v4(),
            location: input.location,
            dish: input.dish,
            // Convert given GqlDate into timestamp or use current time as fallback
            date: input.date.map(Into::into).unwrap_or(chrono::Utc::now()),
            kj: input.kj,
            kcal: input.kcal,
            fat: input.fat,
            saturated_fat: input.saturated_fat,
            carbohydrates: input.carbohydrates,
            sugar: input.sugar,
            fiber: input.fiber,
            protein: input.protein,
            salt: input.salt,
            price_student: input.price_student,
            price_staff: input.price_staff,
            price_guest: input.price_guest,
            not_available_after: None,
            // NOTE: This value currently is unused, but kept here (for now) for backwards compatibility
            status: "AWAITING_APPROVAL".to_string(),
        };

        // Add occurrence
        let insert_result: DbOccurrence = diesel::insert_into(occurrences::table)
            .values(&new_occurrence)
            .returning(DbOccurrence::as_returning())
            .get_result(conn)
            .expect("Error saving new occurrence");

        // Query inserted occurrence
        // NOTE: We cannot simply use the given input as it only contains references for
        // location, dish, side_dishes and tags.
        // Also we do not necessarily know the database defaults for nullable fields.
        let result = occurrences::table
            .filter(occurrences::id.eq(insert_result.id))
            .select(DbOccurrence::as_select())
            .first(conn)
            .expect("Error loading inserted occurrence");

        Ok(result.into())
    }

    async fn update_occurrence(
        &self,
        ctx: &Context<'_>,
        input: UpdateOccurrenceInput,
    ) -> Result<GqlOccurrence> {
        // Get DB connection
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        // Convert the input to a changeset
        let changeset: DbOccurrenceChangeset = input.into();
        // Create query to update the given occurrence
        diesel::update(occurrences::table)
            .filter(occurrences::id.eq(input.id))
            .set(changeset)
            .execute(conn)
            .optional_empty_changeset()
            .expect("Error while updating");

        // Query updated occurrence
        let result = occurrences::table
            .filter(occurrences::id.eq(input.id))
            .select(DbOccurrence::as_select())
            .first(conn)
            .expect("Error loading inserted occurrence");

        Ok(result.into())
    }
}
