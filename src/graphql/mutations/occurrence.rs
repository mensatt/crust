use async_graphql::{dataloader::DataLoader, Context, InputObject, Result};
use diesel::prelude::*;
use diesel::result::Error::NotFound;
use log::warn;

use crate::auth::AuthContext;
use crate::graphql::error::GqlApiError;
use crate::graphql::util::get_conn_from_ctx;
use crate::{
    db::models::{
        occurrence::{DbOccurrence, DbOccurrenceChangeset},
        occurrence_side_dish::DbOccurrenceSideDish,
        occurrence_tag::DbOccurrenceTag,
        tag::DbTag,
    },
    graphql::{
        queries::{GqlDish, GqlOccurrence, GqlTag},
        util::GqlDate,
    },
    schema::{occurrences, occurrences_side_dishes, occurrences_tags, tags},
    DishLoader, OccurrenceLoader,
};

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
    pub side_dishes: Option<Vec<uuid::Uuid>>,
    pub tags: Option<Vec<String>>,
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

#[derive(Debug, InputObject)]
pub struct DeleteOccurrenceInput {
    id: uuid::Uuid,
}

// TODO: I (Bene) think this type is unnecessary and can be replaced by simply returning the
//       Occurrence instead of this wrapper type. Same goes for OccurrenceTag.
//       It currently is present to be as compatible with the existing API as possible.
pub struct OccurrenceSideDish {
    // Internal fields, these should not be exposed via the API
    pub occurrence_id: uuid::Uuid,
    pub dish_id: uuid::Uuid,
}

#[async_graphql::Object]
impl OccurrenceSideDish {
    async fn occurrence(&self, ctx: &Context<'_>) -> Result<GqlOccurrence> {
        let loader = ctx.data::<DataLoader<OccurrenceLoader>>().map_err(|e| {
            GqlApiError::internal("Unable to get OccurrenceLoader from context", e.message)
        })?;
        let occ = loader
            .load_one(self.occurrence_id)
            .await
            .map_err(|e| {
                GqlApiError::internal(
                    format!(
                        "Unable to load occurrence with ID '{}' within occurrence side dish via occurrence loader",
                        self.occurrence_id
                    ),
                    e.message,
                )
            })?
            .ok_or_else(|| {
                GqlApiError::not_found(format!("Occurrence with ID '{}' not found", self.occurrence_id))
            })?;

        Ok(occ.into())
    }

    async fn dish(&self, ctx: &Context<'_>) -> Result<GqlDish> {
        let loader = ctx.data::<DataLoader<DishLoader>>().map_err(|e| {
            GqlApiError::internal("Unable to get DishLoader from context", e.message)
        })?;

        let dish = loader
            .load_one(self.dish_id)
            .await
            .map_err(|e| {
                GqlApiError::internal(
                    format!(
                        "Unable to load dish with ID '{}' within occurrence side dish via dish loader",
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

pub struct OccurrenceTag {
    // Internal fields, these should not be exposed via the API
    pub occurrence_id: uuid::Uuid,
    pub tag_id: String,
}

#[async_graphql::Object]
impl OccurrenceTag {
    async fn occurrence(&self, ctx: &Context<'_>) -> Result<GqlOccurrence> {
        let loader = ctx.data::<DataLoader<OccurrenceLoader>>().map_err(|e| {
            GqlApiError::internal("Unable to get OccurrenceLoader from context", e.message)
        })?;
        let occ = loader
            .load_one(self.occurrence_id)
            .await.map_err(|e| {
                GqlApiError::internal(
                    format!(
                        "Unable to load occurrence with ID '{}' within occurrence tag via occurrence loader",
                        self.occurrence_id
                    ),
                    e.message,
                )
            })?
            .ok_or_else(|| {
                GqlApiError::not_found(format!("Occurrence with ID '{}' not found", self.occurrence_id))
            })?;
        Ok(occ.into())
    }

    async fn tag(&self, ctx: &Context<'_>) -> Result<GqlTag> {
        // Get DB connection
        let conn = &mut get_conn_from_ctx(ctx)?;

        let tag = tags::table
            .filter(tags::key.eq(&self.tag_id))
            .first::<DbTag>(conn)
            .map_err(|e| match e {
                NotFound => {
                    GqlApiError::not_found(format!("Tag with ID '{}' not found", self.tag_id))
                }
                _ => GqlApiError::internal(
                    format!(
                        "Error while querying tag with ID '{}' for occurrence ID '{}'",
                        self.tag_id, self.occurrence_id
                    ),
                    e.to_string(),
                ),
            })?;
        Ok(tag.into())
    }
}

#[derive(Debug, InputObject, Insertable)]
#[diesel(table_name = crate::schema::occurrences_side_dishes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AddSideDishToOccurrenceInput {
    pub occurrence: uuid::Uuid,
    pub dish: uuid::Uuid,
}

#[derive(Debug, InputObject, Identifiable)]
#[diesel(table_name = crate::schema::occurrences_side_dishes)]
#[diesel(primary_key(occurrence, dish))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RemoveSideDishFromOccurrenceInput {
    pub occurrence: uuid::Uuid,
    pub dish: uuid::Uuid,
}

#[derive(Debug, InputObject, Insertable)]
#[diesel(table_name = crate::schema::occurrences_tags)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AddTagToOccurrenceInput {
    pub occurrence: uuid::Uuid,
    pub tag: String,
}

#[derive(Debug, InputObject, Identifiable)]
#[diesel(table_name = crate::schema::occurrences_tags)]
#[diesel(primary_key(occurrence, tag))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RemoveTagFromOccurrenceInput {
    pub occurrence: uuid::Uuid,
    pub tag: String,
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
        // Require authentication for this mutation
        ctx.data::<AuthContext>()?.require_auth()?;

        // Get DB connection
        let conn = &mut get_conn_from_ctx(ctx)?;

        // Convert given GqlDate into timestamp or use current time as fallback
        let date = input.date.map(Into::into).unwrap_or_else(|| {
                warn!(
                    "Unable to convert GqlDate '{:?}' to chrono date. Using current date and time as fallback",
                    input.date
                );
            chrono::Utc::now()
        });

        let new_occurrence = DbOccurrence {
            id: uuid::Uuid::new_v4(),
            location: input.location,
            dish: input.dish,
            date,
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

        // Add occurrence and return it
        // NOTE: We cannot simply return the given input as we do not
        // necessarily know the database defaults for nullable fields.
        let result = diesel::insert_into(occurrences::table)
            .values(&new_occurrence)
            .get_result::<DbOccurrence>(conn)
            // NOTE: In theory .get_result() could return NotFound, but if that happens on insert
            //       something internally has gone wrong.
            .map_err(|e| {
                GqlApiError::internal("Error while inserting new occurrence", e.to_string())
            })?;

        // If present, insert side dishes for occurrence
        if let Some(side_dish_ids) = input.side_dishes {
            for side_dish_id in side_dish_ids {
                diesel::insert_into(occurrences_side_dishes::table)
                    .values(&DbOccurrenceSideDish {
                        occurrence: new_occurrence.id,
                        dish: side_dish_id,
                    })
                    .execute(conn)
                    .map_err(|e| {
                        GqlApiError::internal(
                            "Error while inserting side dishes for new occurrence",
                            e.to_string(),
                        )
                    })?;
            }
        }

        // If present, insert tags for occurrence
        if let Some(tags) = input.tags {
            for tag in tags {
                diesel::insert_into(occurrences_tags::table)
                    .values(&DbOccurrenceTag {
                        occurrence: new_occurrence.id,
                        tag,
                    })
                    .execute(conn)
                    .map_err(|e| {
                        GqlApiError::internal(
                            "Error while inserting tags for new occurrence",
                            e.to_string(),
                        )
                    })?;
            }
        }

        Ok(result.into())
    }

    async fn update_occurrence(
        &self,
        ctx: &Context<'_>,
        input: UpdateOccurrenceInput,
    ) -> Result<GqlOccurrence> {
        // Require authentication for this mutation
        ctx.data::<AuthContext>()?.require_auth()?;

        // Get DB connection
        let conn = &mut get_conn_from_ctx(ctx)?;

        // Save occurrence id for later and convert the input to a changeset
        let occ_id = input.id;
        let changeset: DbOccurrenceChangeset = input.into();

        // Try to update, map empty changeset to None (instead of Error)
        let pot_empty_changeset = diesel::update(occurrences::table)
            .filter(occurrences::id.eq(occ_id))
            .set(changeset)
            .get_result::<DbOccurrence>(conn)
            .optional_empty_changeset()
            .map_err(|e| match e {
                NotFound => {
                    GqlApiError::not_found(format!("Occurrence with ID '{}' not found", input.id))
                }
                _ => GqlApiError::internal(
                    format!("Error while updating occurrence with ID '{}'", input.id),
                    e.to_string(),
                ),
            })?;

        // Use non-empty changeset if present and fall back to querying otherwise
        let result = match pot_empty_changeset {
            Some(occurrence) => occurrence,
            // Fallback query that returns the occurrence as it is stored in the database
            None => occurrences::table
                .filter(occurrences::id.eq(occ_id))
                .select(DbOccurrence::as_select())
                .first(conn)
                .map_err(|e| match e {
                    NotFound => GqlApiError::not_found(format!(
                        "Occurrence with ID '{}' not found",
                        input.id
                    )),
                    _ => GqlApiError::internal(
                        format!("Error while updating occurrence with ID '{}'", input.id),
                        e.to_string(),
                    ),
                })?,
        };

        Ok(result.into())
    }

    async fn delete_occurrence(
        &self,
        ctx: &Context<'_>,
        input: DeleteOccurrenceInput,
        // TODO: Consider other response type
        //       Number of rows affected?, id of deleted object?, Query object before deletion?
    ) -> Result<bool> {
        // Require authentication for this mutation
        ctx.data::<AuthContext>()?.require_auth()?;

        // Get DB connection
        let conn = &mut get_conn_from_ctx(ctx)?;

        let amount = diesel::delete(occurrences::table)
            .filter(occurrences::id.eq(input.id))
            .execute(conn)
            .map_err(|e| {
                GqlApiError::internal(
                    format!("Error while deleting occurrence with ID '{}'", input.id),
                    e.to_string(),
                )
            })?;

        Ok(amount == 1)
    }

    async fn add_side_dish_to_occurrence(
        &self,
        ctx: &Context<'_>,
        input: AddSideDishToOccurrenceInput,
    ) -> Result<OccurrenceSideDish> {
        // Require authentication for this mutation
        ctx.data::<AuthContext>()?.require_auth()?;

        // Get DB connection
        let conn = &mut get_conn_from_ctx(ctx)?;

        // Add side dish
        diesel::insert_into(occurrences_side_dishes::table)
            .values(&input)
            .execute(conn)
            .map_err(|e| {
                GqlApiError::internal(
                    format!(
                        "Error while adding side dish with ID '{}' to occurrence with ID '{}'",
                        input.dish, input.occurrence
                    ),
                    e.to_string(),
                )
            })?;

        Ok(OccurrenceSideDish {
            occurrence_id: input.occurrence,
            dish_id: input.dish,
        })
    }

    async fn remove_side_dish_from_occurrence(
        &self,
        ctx: &Context<'_>,
        input: RemoveSideDishFromOccurrenceInput,
    ) -> Result<OccurrenceSideDish> {
        // Require authentication for this mutation
        ctx.data::<AuthContext>()?.require_auth()?;

        // Get DB connection
        let conn = &mut get_conn_from_ctx(ctx)?;

        // Identify...
        let rows_to_delete = occurrences_side_dishes::table.find((input.occurrence, input.dish));
        // ... and delete rows
        diesel::delete(rows_to_delete).execute(conn).map_err(|e| {
            GqlApiError::internal(
                format!(
                    "Error while removing side dish with ID '{}' from occurrence with ID '{}'",
                    input.dish, input.occurrence
                ),
                e.to_string(),
            )
        })?;

        Ok(OccurrenceSideDish {
            occurrence_id: input.occurrence,
            dish_id: input.dish,
        })
    }

    async fn add_tag_to_occurrence(
        &self,
        ctx: &Context<'_>,
        input: AddTagToOccurrenceInput,
    ) -> Result<OccurrenceTag> {
        // Require authentication for this mutation
        ctx.data::<AuthContext>()?.require_auth()?;

        // Get DB connection
        let conn = &mut get_conn_from_ctx(ctx)?;

        // Add tag for occurrence
        diesel::insert_into(occurrences_tags::table)
            .values(&input)
            .execute(conn)
            .map_err(|e| {
                GqlApiError::internal(
                    format!(
                        "Error while adding tag '{}' to occurrence with ID '{}'",
                        input.tag, input.occurrence
                    ),
                    e.to_string(),
                )
            })?;

        Ok(OccurrenceTag {
            occurrence_id: input.occurrence,
            tag_id: input.tag,
        })
    }

    async fn remove_tag_from_occurrence(
        &self,
        ctx: &Context<'_>,
        input: RemoveTagFromOccurrenceInput,
    ) -> Result<OccurrenceTag> {
        // Require authentication for this mutation
        ctx.data::<AuthContext>()?.require_auth()?;

        // Get DB connection
        let conn = &mut get_conn_from_ctx(ctx)?;

        // Identify...
        let rows_to_delete = occurrences_tags::table.find((input.occurrence, &input.tag));
        // ... and delete rows
        diesel::delete(rows_to_delete).execute(conn).map_err(|e| {
            GqlApiError::internal(
                format!(
                    "Error while removing tag '{}' from occurrence with ID '{}'",
                    input.tag, input.occurrence
                ),
                e.to_string(),
            )
        })?;

        Ok(OccurrenceTag {
            occurrence_id: input.occurrence,
            tag_id: input.tag,
        })
    }
}
