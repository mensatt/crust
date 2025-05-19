use async_graphql::dataloader::DataLoader;
use async_graphql::{Context, InputObject, Result};
use diesel::prelude::*;
use diesel::result::Error::NotFound;

use crate::auth::AuthContext;
use crate::graphql::dataloaders::{ReviewLoader, ReviewLoaderKey};
use crate::graphql::error::GqlApiError;
use crate::graphql::queries::GqlReview;
use crate::graphql::util::get_conn_from_ctx;
use crate::schema::reviews;
use crate::{
    db::models::{
        image::DbImage,
        review::{DbReview, DbReviewChangeset},
    },
    schema::images,
};

#[derive(Debug, InputObject)]
pub struct ImageInput {
    id: uuid::Uuid,
    // TODO: This is currently unused; Implement functionality from old backend
    rotation: Option<i64>,
}

#[derive(Debug, InputObject)]
pub struct CreateReviewInput {
    pub occurrence: uuid::Uuid,
    pub display_name: Option<String>,
    pub stars: i64,
    pub text: Option<String>,
    pub images: Option<Vec<ImageInput>>,
}

#[derive(Debug, InputObject, Clone)]
pub struct UpdateReviewInput {
    pub id: uuid::Uuid,
    pub occurrence: Option<uuid::Uuid>,
    pub display_name: Option<String>,
    pub stars: Option<i64>,
    pub text: Option<String>,
    pub approved: Option<bool>, // If this is present, approved_at will be set
}

impl From<UpdateReviewInput> for DbReviewChangeset {
    fn from(value: UpdateReviewInput) -> Self {
        let now = chrono::Utc::now();

        // Check if any of the optional fields is present
        // Some(<field>) means the field will be updated in the DB
        // NOTE: If the UpdateReviewInput is extended with additional optional fields
        //       they have to be added here
        let has_updates = [
            value.display_name.is_some(),
            value.stars.is_some(),
            value.text.is_some(),
            value.occurrence.is_some(),
            value.approved.is_some(),
        ]
        .iter()
        .any(|&b| b);

        // Only set the update timestamp if any "real" value is going to be changed
        let updated_at = has_updates.then_some(now);
        // Set approved timestamp if approved was passed in the mutation
        let accepted_at = (value.approved == Some(true)).then_some(now);
        DbReviewChangeset {
            display_name: value.display_name,
            stars: value.stars,
            text: value.text,
            occurrence: value.occurrence,
            updated_at,
            accepted_at,
            // Optional fields, that are unused by GraphQL
            created_at: None,
        }
    }
}

#[derive(Debug, InputObject)]
pub struct DeleteReviewInput {
    id: uuid::Uuid,
}

#[derive(Debug, InputObject)]
pub struct AddImagesToReviewInput {
    pub review: uuid::Uuid,
    pub images: Vec<ImageInput>,
}

#[derive(Debug, InputObject)]
pub struct RemoveImagesFromReviewInput {
    pub review: uuid::Uuid,
    pub images: Vec<uuid::Uuid>,
}

#[derive(Default)]
pub struct ReviewMutations;

#[async_graphql::Object]
impl ReviewMutations {
    async fn create_review(
        &self,
        ctx: &Context<'_>,
        input: CreateReviewInput,
    ) -> Result<GqlReview> {
        // NOTE: No authentication on this mutation, as all users shall be able to create reviews

        // Get DB connection
        let conn = &mut get_conn_from_ctx(ctx)?;

        let now = chrono::Utc::now();
        let new_review = DbReview {
            id: uuid::Uuid::new_v4(),
            occurrence: input.occurrence,
            display_name: input.display_name,
            stars: input.stars,
            text: input.text,
            updated_at: now,
            created_at: now,
            accepted_at: None,
        };

        // TODO: Utilize transactions here to ensure atomic adding of review + images?

        // Add review and return it
        let result: DbReview = diesel::insert_into(reviews::table)
            .values(&new_review)
            .get_result(conn)
            // NOTE: In theory .get_result() could return NotFound, but if that happens on insert
            //       something internally has gone wrong.
            .map_err(|e| {
                GqlApiError::internal("Error while inserting new review", e.to_string())
            })?;

        // If present, create images for review
        if let Some(images) = input.images {
            // TODO: Handle image rotation (?)
            // TODO: Notify image service about submitted image
            for image in images {
                diesel::insert_into(images::table)
                    .values(&DbImage {
                        id: image.id,
                        review: new_review.id,
                    })
                    .execute(conn)
                    .map_err(|e| {
                        GqlApiError::internal(
                            "Error while inserting image for new review",
                            e.to_string(),
                        )
                    })?;
            }
        }

        Ok(result.into())
    }

    async fn update_review(
        &self,
        ctx: &Context<'_>,
        input: UpdateReviewInput,
    ) -> Result<GqlReview> {
        // Require authentication for this mutation
        ctx.data::<AuthContext>()?.require_auth()?;

        // Get DB connection
        let conn = &mut get_conn_from_ctx(ctx)?;

        // Save review_id for later and convert the input to a changeset
        let review_id = input.id;
        let changeset: DbReviewChangeset = input.into();

        // Try to update, map empty changeset to None (instead of Error)
        let pot_empty_changeset: Option<DbReview> = diesel::update(reviews::table)
            .filter(reviews::id.eq(review_id))
            .set(changeset)
            .get_result::<DbReview>(conn)
            .optional_empty_changeset()
            .map_err(|e| match e {
                NotFound => {
                    GqlApiError::not_found(format!("Review with ID '{}' not found", review_id))
                }
                _ => GqlApiError::internal(
                    format!("Error while updating review with ID '{}'", review_id),
                    e.to_string(),
                ),
            })?;

        // Use non-empty changeset if present and fall back to querying otherwise
        let result = match pot_empty_changeset {
            Some(review) => review,
            // Fallback query that returns the review as it is stored in the database
            None => reviews::table
                .filter(reviews::id.eq(review_id))
                .select(DbReview::as_select())
                .first(conn)
                .map_err(|e| match e {
                    NotFound => {
                        GqlApiError::not_found(format!("Review with ID '{}' not found", review_id))
                    }
                    _ => GqlApiError::internal(
                        format!("Error while updating review with ID '{}'", review_id),
                        e.to_string(),
                    ),
                })?,
        };

        Ok(result.into())
    }

    async fn delete_review(
        &self,
        ctx: &Context<'_>,
        input: DeleteReviewInput,
        // TODO: Consider other response type
        //       Number of rows affected?, id of deleted object?, Query object before deletion?
    ) -> Result<bool> {
        // Require authentication for this mutation
        ctx.data::<AuthContext>()?.require_auth()?;

        // Get DB connection
        let conn = &mut get_conn_from_ctx(ctx)?;

        let amount = diesel::delete(reviews::table)
            .filter(reviews::id.eq(input.id))
            .execute(conn)
            .map_err(|e| {
                GqlApiError::internal(
                    format!("Error while deleting review with ID '{}'", input.id),
                    e.to_string(),
                )
            })?;
        Ok(amount == 1)
    }

    async fn add_images_to_review(
        &self,
        ctx: &Context<'_>,
        input: AddImagesToReviewInput,
    ) -> Result<GqlReview> {
        // NOTE: No auth on this mutation; all users shall be able to create reviews (with images)

        // Get DB connection
        let conn = &mut get_conn_from_ctx(ctx)?;

        // Create images for review
        // TODO: Handle image rotation (?)
        // TODO: Notify image service about submitted image(s)
        for image in input.images {
            diesel::insert_into(images::table)
                .values(&DbImage {
                    id: image.id,
                    review: input.review,
                })
                .execute(conn)
                .map_err(|e| {
                    GqlApiError::internal(
                        format!(
                            "Error while adding image with ID '{}' to review with ID '{}'",
                            image.id, input.review
                        ),
                        e.to_string(),
                    )
                })?;
        }

        // Load and return review
        let loader = ctx.data::<DataLoader<ReviewLoader>>().map_err(|e| {
            GqlApiError::internal("Unable to get ReviewLoader from context", e.message)
        })?;
        let rev = loader
            .load_one(ReviewLoaderKey::ByReviewId { id: input.review })
            .await
            .map_err(|e| {
                GqlApiError::internal(
                    format!(
                        "Unable to load review with ID '{}' via review loader",
                        input.review
                    ),
                    e.message,
                )
            })?
            .and_then(|v| v.into_iter().next())
            .ok_or_else(|| {
                GqlApiError::not_found(format!("Review with ID '{}' not found", input.review))
            })?;
        Ok(rev.into())
    }

    async fn remove_images_from_review(
        &self,
        ctx: &Context<'_>,
        input: RemoveImagesFromReviewInput,
    ) -> Result<GqlReview> {
        // NOTE: This mutation was/is not authenticated in the old backend.
        // TODO: Reconsider if this is sane

        // Get DB connection
        let conn = &mut get_conn_from_ctx(ctx)?;

        // Delete images for review
        diesel::delete(
            images::table.filter(
                images::review
                    .eq(input.review)
                    .and(images::id.eq_any(input.images)),
            ),
        )
        .execute(conn)
        .map_err(|e| {
            GqlApiError::internal(
                format!(
                    "Error while removing image(s) from review with ID '{}'",
                    input.review
                ),
                e.to_string(),
            )
        })?;

        // Load and return review
        let loader = ctx.data::<DataLoader<ReviewLoader>>().map_err(|e| {
            GqlApiError::internal("Unable to get ReviewLoader from context", e.message)
        })?;
        let rev = loader
            .load_one(ReviewLoaderKey::ByReviewId { id: input.review })
            .await
            .map_err(|e| {
                GqlApiError::internal(
                    format!(
                        "Unable to load review with ID '{}' via review loader",
                        input.review
                    ),
                    e.message,
                )
            })?
            .and_then(|v| v.into_iter().next())
            .ok_or_else(|| {
                GqlApiError::not_found(format!("Review with ID '{}' not found", input.review))
            })?;
        Ok(rev.into())
    }
}
