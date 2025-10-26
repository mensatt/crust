use async_graphql::dataloader::DataLoader;
use async_graphql::{Context, InputObject, Result};
use diesel::prelude::*;
use diesel::result::Error::NotFound;
use log::error;

use crate::auth::AuthContext;
use crate::graphql::dataloaders::{ReviewLoader, ReviewLoaderKey};
use crate::graphql::error::GqlApiError;
use crate::graphql::queries::GqlReview;
use crate::graphql::subscriptions::{ReviewEvent, SubscriptionBroker};
use crate::graphql::util::get_conn_from_ctx;
use crate::image_service::ImageServiceClient;
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
        let accepted_at = match value.approved {
            None => None,
            Some(true) => Some(Some(now)),
            Some(false) => Some(None),
        };
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

        // Get image service client
        let image_service = ctx.data::<ImageServiceClient>().map_err(|e| {
            GqlApiError::internal("Unable to get ImageServiceClient from context", e.message)
        })?;

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

        // Start transaction
        let result: DbReview = conn
            .transaction(|conn| {
                // Insert the review
                let review = diesel::insert_into(reviews::table)
                    .values(&new_review)
                    .get_result::<DbReview>(conn)?;

                Ok::<DbReview, diesel::result::Error>(review)
            })
            .map_err(|e| {
                GqlApiError::internal("Error while inserting new review", e.to_string())
            })?;

        // Process & store images (if present)
        // Submit images to the image service BEFORE committing them to the database
        if let Some(images) = input.images {
            let submitted_images = image_service.submit_images(images.iter().map(|img| img.id).collect()).await;

            // Handle rotation for successfully submitted images
            for image in &images {
                if let Some(rotation) = image.rotation {
                    if submitted_images.contains(&image.id) {
                        if let Err(e) = image_service.rotate_image(image.id, rotation).await {
                            error!("Failed to rotate image {}: {}", image.id, e);
                            // Continue despite rotation failure
                        }
                    }
                }
            }

            // Now store successfully submitted images in the database using a transaction
            conn.transaction(|conn| {
                for image_id in submitted_images {
                    diesel::insert_into(images::table)
                        .values(&DbImage {
                            id: image_id,
                            review: result.id,
                        })
                        .execute(conn)
                        .ok(); // Continue if one image fails to store
                }
                Ok::<(), diesel::result::Error>(())
            })
            .map_err(|e| {
                GqlApiError::internal("Error while inserting images for new review", e.to_string())
            })?;
        }

        let gql_review: GqlReview = result.into();

        // Publish review created event to subscribers
        if let Ok(broker) = ctx.data::<SubscriptionBroker>() {
            broker.publish_review(ReviewEvent::Created(gql_review.clone()));
        }

        Ok(gql_review)
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

        // Get image service client
        let image_service = ctx.data::<ImageServiceClient>().map_err(|e| {
            GqlApiError::internal("Unable to get ImageServiceClient from context", e.message)
        })?;

        // Save review_id for later and convert the input to a changeset
        let review_id = input.id;
        let input_approved = input.approved;
        let mut changeset: DbReviewChangeset = input.into();

        // Query the review before update to check current state (outside transaction)
        let pre_update_review = reviews::table
            .filter(reviews::id.eq(review_id))
            .select(DbReview::as_select())
            .first(conn)
            .map_err(|e| match e {
                NotFound => {
                    GqlApiError::not_found(format!("Review with ID '{}' not found", review_id))
                }
                _ => GqlApiError::internal(
                    format!("Error while querying review with ID '{}'", review_id),
                    e.to_string(),
                ),
            })?;

        let old_accepted_at = pre_update_review.accepted_at;

        // If input.approved is set and true, check if already approved
        // if so, don't change accepted_at (aka set it to None in the changeset)
        if changeset.accepted_at.flatten().is_some() && old_accepted_at.is_some() {
            changeset.accepted_at = None
        }

        // Update the review in a transaction
        let updated_review = conn
            .transaction::<DbReview, diesel::result::Error, _>(|conn| {
                diesel::update(reviews::table)
                    .filter(reviews::id.eq(review_id))
                    .set(&changeset)
                    .get_result::<DbReview>(conn)
                    .optional_empty_changeset()
                    .map(|opt| opt.unwrap_or(pre_update_review.clone()))
            })
            .map_err(|e| match e {
                NotFound => {
                    GqlApiError::not_found(format!("Review with ID '{}' not found", review_id))
                }
                _ => GqlApiError::internal(
                    format!("Error while updating review with ID '{}'", review_id),
                    e.to_string(),
                ),
            })?;

        // Check if review was approved for the first time
        let first_approval = input_approved == Some(true) && old_accepted_at.is_none();
        // Check if review was unapproved
        let unapproval = input_approved == Some(false) && old_accepted_at.is_some();

        // Handle image approval if review was approved for the first time
        if first_approval {
            // Query all images of the review
            let image_ids: Vec<uuid::Uuid> = images::table
                .filter(images::review.eq(review_id))
                .select(images::id)
                .load(conn)
                .map_err(|e| {
                    GqlApiError::internal(
                        format!("Error while querying images for review with ID '{}'", review_id),
                        e.to_string(),
                    )
                })?;

            if !image_ids.is_empty() {
                // Approve all images in the image service
                match image_service.approve_images(image_ids.clone()).await {
                    Ok(_approved_images) => {
                        // Success - images are approved
                    }
                    Err(e) => {
                        error!("Failed to approve images for review {}: {}", review_id, e);
                        // Rollback the database change
                        conn.transaction(|conn| {
                            diesel::update(reviews::table)
                                .filter(reviews::id.eq(review_id))
                                .set(reviews::accepted_at.eq::<Option<chrono::DateTime<chrono::Utc>>>(None))
                                .execute(conn)
                        })
                        .ok(); // Ignore rollback errors

                        return Err(GqlApiError::internal(
                            format!("Failed to approve images for review with ID '{}'", review_id),
                            e.to_string(),
                        )
                        .into());
                    }
                }
            }
        }

        // Handle image unapproval if review was unapproved
        if unapproval {
            // Query all images of the review
            let image_ids: Vec<uuid::Uuid> = images::table
                .filter(images::review.eq(review_id))
                .select(images::id)
                .load(conn)
                .map_err(|e| {
                    GqlApiError::internal(
                        format!("Error while querying images for review with ID '{}'", review_id),
                        e.to_string(),
                    )
                })?;

            if !image_ids.is_empty() {
                // Unapprove all images in the image service
                match image_service.unapprove_images(image_ids.clone()).await {
                    Ok(_unapproved_images) => {
                        // Success - images are unapproved
                    }
                    Err(e) => {
                        error!("Failed to unapprove images for review {}: {}", review_id, e);
                        // Rollback the database change by re-approving
                        conn.transaction(|conn| {
                            diesel::update(reviews::table)
                                .filter(reviews::id.eq(review_id))
                                .set(reviews::accepted_at.eq(old_accepted_at))
                                .execute(conn)
                        })
                        .ok(); // Ignore rollback errors

                        return Err(GqlApiError::internal(
                            format!("Failed to unapprove images for review with ID '{}'", review_id),
                            e.to_string(),
                        )
                        .into());
                    }
                }
            }
        }

        let gql_review: GqlReview = updated_review.into();

        // Publish review accepted event to subscribers if the review was approved
        if first_approval {
            if let Ok(broker) = ctx.data::<SubscriptionBroker>() {
                broker.publish_review(ReviewEvent::Accepted(gql_review.clone()));
            }
        }

        Ok(gql_review)
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

        // Get image service client
        let image_service = ctx.data::<ImageServiceClient>().map_err(|e| {
            GqlApiError::internal("Unable to get ImageServiceClient from context", e.message)
        })?;

        // Save image UUIDs as the images will be deleted by deleting the review (cascaded)
        let image_ids: Vec<uuid::Uuid> = images::table
            .filter(images::review.eq(input.id))
            .select(images::id)
            .load(conn)
            .map_err(|e| {
                GqlApiError::internal(
                    format!("Error while querying images for review with ID '{}'", input.id),
                    e.to_string(),
                )
            })?;

        // Remove the images from the DB (cascaded via review) before deleting from the image service
        let amount = conn
            .transaction(|conn| {
                diesel::delete(reviews::table)
                    .filter(reviews::id.eq(input.id))
                    .execute(conn)
            })
            .map_err(|e| {
                GqlApiError::internal(
                    format!("Error while deleting review with ID '{}'", input.id),
                    e.to_string(),
                )
            })?;

        // Delete images from the image service
        if !image_ids.is_empty() {
            let deleted_images = image_service.delete_images(image_ids.clone()).await;

            if deleted_images.len() != image_ids.len() {
                error!(
                    "Failed to delete all images from image service for review {}: expected {}, got {}",
                    input.id,
                    image_ids.len(),
                    deleted_images.len()
                );
                // Note: In the old backend, this would return an error
                // We're being more lenient here and just logging
            }
        }

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

        // Get image service client
        let image_service = ctx.data::<ImageServiceClient>().map_err(|e| {
            GqlApiError::internal("Unable to get ImageServiceClient from context", e.message)
        })?;

        // Submit images to the image service BEFORE adding them to the database
        let submitted_images = image_service.submit_images(input.images.iter().map(|img| img.id).collect()).await;

        // Handle rotation for successfully submitted images
        for image in &input.images {
            if let Some(rotation) = image.rotation {
                if submitted_images.contains(&image.id) {
                    if let Err(e) = image_service.rotate_image(image.id, rotation).await {
                        error!("Failed to rotate image {}: {}", image.id, e);
                        // Continue despite rotation failure
                    }
                }
            }
        }

        // Now store successfully submitted images in the database using a transaction
        conn.transaction(|conn| {
            for image_id in &submitted_images {
                diesel::insert_into(images::table)
                    .values(&DbImage {
                        id: *image_id,
                        review: input.review,
                    })
                    .execute(conn)
                    .ok(); // Continue if one image fails to store
            }
            Ok::<(), diesel::result::Error>(())
        })
        .map_err(|e| {
            GqlApiError::internal(
                format!("Error while adding images to review with ID '{}'", input.review),
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

    async fn remove_images_from_review(
        &self,
        ctx: &Context<'_>,
        input: RemoveImagesFromReviewInput,
    ) -> Result<GqlReview> {
        // NOTE: This mutation was/is not authenticated in the old backend.
        // TODO: Reconsider if this is sane

        // Get DB connection
        let conn = &mut get_conn_from_ctx(ctx)?;

        // Get image service client
        let image_service = ctx.data::<ImageServiceClient>().map_err(|e| {
            GqlApiError::internal("Unable to get ImageServiceClient from context", e.message)
        })?;

        // Delete images from the database using a transaction
        conn.transaction(|conn| {
            diesel::delete(
                images::table.filter(
                    images::review
                        .eq(input.review)
                        .and(images::id.eq_any(&input.images)),
                ),
            )
            .execute(conn)
        })
        .map_err(|e| {
            GqlApiError::internal(
                format!(
                    "Error while removing image(s) from review with ID '{}'",
                    input.review
                ),
                e.to_string(),
            )
        })?;

        // Delete images from the image service
        let deleted_images = image_service.delete_images(input.images.clone()).await;

        if deleted_images.len() != input.images.len() {
            error!(
                "Failed to delete all images from image service for review {}: expected {}, got {}",
                input.review,
                input.images.len(),
                deleted_images.len()
            );
            // Note: Old backend would return an error here
            // We're being more lenient and just logging
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
}
