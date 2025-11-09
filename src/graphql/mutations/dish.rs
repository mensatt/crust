use async_graphql::{Context, InputObject, Result};
use diesel::prelude::*;
use diesel::result::Error::NotFound;
use uuid::Uuid;

use crate::auth::AuthContext;
use crate::db::models::dish::{DbDish, DbDishChangeset};
use crate::graphql::error::GqlApiError;
use crate::graphql::queries::GqlDish;
use crate::graphql::util::get_conn_from_ctx;
use crate::schema::{dishes, dishes_aliases, occurrences, occurrences_side_dishes};

#[derive(Debug, InputObject)]
pub struct CreateDishInput {
    pub name_de: String,
    pub name_en: Option<String>,
}

#[derive(Debug, InputObject, AsChangeset)]
#[diesel(table_name = crate::schema::dishes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UpdateDishInput {
    pub id: Uuid,
    pub name_de: Option<String>,
    pub name_en: Option<String>,
}

#[derive(Debug, InputObject)]
pub struct MergeDishesInput {
    pub name_de: String,
    pub name_en: Option<String>,
    pub merge_ids: Vec<Uuid>,
}

#[derive(Default)]
pub struct DishMutations;

#[async_graphql::Object]
impl DishMutations {
    pub async fn create_dish(&self, ctx: &Context<'_>, input: CreateDishInput) -> Result<GqlDish> {
        // Require authentication for this mutation
        ctx.data::<AuthContext>()?.require_auth()?;

        // Get DB connection
        let conn = &mut get_conn_from_ctx(ctx)?;

        // Construct new dish
        let new_dish = DbDish {
            id: Uuid::new_v4(),
            name_de: input.name_de,
            name_en: input.name_en,
        };

        // Add dish and return it
        let results: DbDish = diesel::insert_into(dishes::table)
            .values(&new_dish)
            .get_result(conn)
            // NOTE: In theory .get_result() could return NotFound, but if that happens on insert
            //       something internally has gone wrong.
            .map_err(|e| GqlApiError::internal("Error while inserting new dish", e.to_string()))?;

        Ok(results.into())
    }

    async fn update_dish(&self, ctx: &Context<'_>, input: UpdateDishInput) -> Result<GqlDish> {
        // Require authentication for this mutation
        ctx.data::<AuthContext>()?.require_auth()?;

        // Get DB connection
        let conn = &mut get_conn_from_ctx(ctx)?;

        // Try to update, map empty changeset to None (instead of Error)
        let pot_empty_changeset = diesel::update(dishes::table)
            .filter(dishes::id.eq(input.id))
            .set(&input)
            .get_result(conn)
            .optional_empty_changeset()
            .map_err(|e| match e {
                NotFound => {
                    GqlApiError::not_found(format!("Dish with ID '{}' not found", input.id))
                }
                _ => GqlApiError::internal(
                    format!("Error while updating dish with ID '{}'", input.id),
                    e.to_string(),
                ),
            })?;

        // Use non-empty changeset if present and fall back to querying otherwise
        let result = match pot_empty_changeset {
            Some(dish) => dish,
            // Fallback query that returns the dish as it is stored in the database
            None => dishes::table
                .filter(dishes::id.eq(input.id))
                .select(DbDish::as_select())
                .first::<DbDish>(conn)
                .map_err(|e| match e {
                    NotFound => {
                        GqlApiError::not_found(format!("Dish with ID '{}' not found", input.id))
                    }
                    _ => GqlApiError::internal(
                        format!("Error while updating dish with ID '{}'", input.id),
                        e.to_string(),
                    ),
                })?,
        };

        Ok(result.into())
    }

    async fn merge_dishes(&self, ctx: &Context<'_>, input: MergeDishesInput) -> Result<GqlDish> {
        // Require authentication for this mutation
        ctx.data::<AuthContext>()?.require_auth()?;

        // Merging less than two dishes makes no sense
        if input.merge_ids.len() < 2 {
            return Err(GqlApiError::invalid_input("Merging needs at least two dish ids").into());
        }

        // Get DB connection
        let conn = &mut get_conn_from_ctx(ctx)?;

        // Check if all given IDs exist in the dishes table
        let existing_dishes = dishes::table
            .filter(dishes::id.eq_any(&input.merge_ids))
            .select(dishes::id)
            .load(conn)
            .map_err(|e| {
                GqlApiError::internal(
                    "Error while querying if to-be-merged dishes exist",
                    e.to_string(),
                )
            })?;

        // If the lengths don't match, some IDs are missing, report them
        if existing_dishes.len() != input.merge_ids.len() {
            let missing_ids: Vec<Uuid> = input
                .merge_ids
                .iter()
                .cloned()
                .filter(|id| !existing_dishes.contains(id))
                .collect();

            return Err(GqlApiError::not_found(format!(
                "The following dish IDs were not found: {:?}",
                missing_ids
            ))
            .into());
        }

        let merged_dish = conn
            .transaction::<DbDish, diesel::result::Error, _>(|conn| {
                // 1. Update or insert dish
                // Check if dish with name_de already exists, if so update it, otherwise insert
                // NOTE: This is the cause if name_de from one of the merge dishes is kept
                let potentially_existing_dish = dishes::table
                    .filter(dishes::name_de.eq(&input.name_de))
                    .select(DbDish::as_select())
                    .first::<DbDish>(conn)
                    .optional()?;

                let upserted_dish = if let Some(dish) = potentially_existing_dish {
                    // Use DbDishChangeset which allows setting name_en (back) to NULL
                    // if it is not given in the query
                    diesel::update(dishes::table.find(dish.id))
                        .set(DbDishChangeset {
                            name_de: None,
                            name_en: Some(input.name_en),
                        })
                        .get_result(conn)?
                } else {
                    let new_dish = DbDish {
                        id: Uuid::new_v4(),
                        name_de: input.name_de,
                        name_en: input.name_en,
                    };
                    diesel::insert_into(dishes::table)
                        .values(&new_dish)
                        .execute(conn)?;
                    new_dish
                };

                // 2. Move all old aliases to new dish
                diesel::update(dishes_aliases::table)
                    .filter(dishes_aliases::dish.eq_any(&input.merge_ids))
                    .set(dishes_aliases::dish.eq(upserted_dish.id))
                    .execute(conn)?;

                // 3. Reassign occurrences to the new dish
                diesel::update(occurrences::table)
                    .filter(occurrences::dish.eq_any(&input.merge_ids))
                    .set(occurrences::dish.eq(upserted_dish.id))
                    .execute(conn)?;

                /* 4. Reassign occurrences_side_dishes
                 * NOTE:
                 *   We cannot simply do an update on the rows that previously existed, as
                 *   the primary key requirement (uniqueness of (occurrence, dish) combination)
                 *   could be violated now that two (previously distinct) dishes are merged.
                 *   In such cases, the update would fail, which we would have to handle separately.
                 *
                 *   It is easier to insert new links with ON CONFLICT DO NOTHING to avoid these
                 *   duplicates - that is what we do here.
                 */

                // 4.1 Query occurrences_side_dishes that have any of old dish id's
                let side_dish_links: Vec<(Uuid, Uuid)> = occurrences_side_dishes::table
                    .filter(occurrences_side_dishes::dish.eq_any(&input.merge_ids))
                    .select((
                        occurrences_side_dishes::occurrence,
                        occurrences_side_dishes::dish,
                    ))
                    .load(conn)?;

                // 4.2 Insert new links between the new dish and all side dishes
                for (occurrence_id, _) in side_dish_links {
                    diesel::insert_into(occurrences_side_dishes::table)
                        .values((
                            occurrences_side_dishes::occurrence.eq(occurrence_id),
                            occurrences_side_dishes::dish.eq(upserted_dish.id),
                        ))
                        .on_conflict((
                            occurrences_side_dishes::occurrence,
                            occurrences_side_dishes::dish,
                        ))
                        .do_nothing()
                        .execute(conn)?;
                }

                // 4.3 Delete side dishes referencing (now obsolete) old dish ids
                diesel::delete(occurrences_side_dishes::table)
                    .filter(occurrences_side_dishes::dish.eq_any(&input.merge_ids))
                    .execute(conn)?;

                // 5. Delete old dishes
                diesel::delete(dishes::table)
                    .filter(
                        dishes::id
                            .eq_any(&input.merge_ids)
                            .and(dishes::id.ne(upserted_dish.id)),
                    )
                    .execute(conn)?;

                Ok(upserted_dish)
            })
            .map_err(|e| {
                GqlApiError::internal(
                    format!(
                        "Error while merging dishes with ids: '{:#?}'",
                        input.merge_ids
                    ),
                    e.to_string(),
                )
            })?;

        Ok(merged_dish.into())
    }
}
