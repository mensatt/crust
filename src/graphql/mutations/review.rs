use async_graphql::{Context, InputObject, Result};
use diesel::prelude::*;

use crate::db::{
    conn::DbPool,
    models::review::{DbReview, DbReviewChangeset},
};
use crate::graphql::queries::GqlReview;
use crate::schema::reviews;

#[derive(Debug, InputObject)]
pub struct CreateReviewInput {
    pub occurrence: uuid::Uuid,
    pub display_name: Option<String>,
    pub stars: i64,
    pub text: Option<String>,
    // TODO: Images
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

#[derive(Default)]
pub struct ReviewMutations;

#[async_graphql::Object]
impl ReviewMutations {
    async fn create_review(
        &self,
        ctx: &Context<'_>,
        input: CreateReviewInput,
    ) -> Result<GqlReview> {
        // Get DB connection
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

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

        // Add review and return it
        let result: DbReview = diesel::insert_into(reviews::table)
            .values(&new_review)
            .get_result(conn)
            .expect("Error saving new review");

        Ok(result.into())
    }

    async fn update_review(
        &self,
        ctx: &Context<'_>,
        input: UpdateReviewInput,
    ) -> Result<GqlReview> {
        // Get DB connection
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        // Save review_id for later and convert the input to a changeset
        let review_id = input.id;
        let changeset: DbReviewChangeset = input.into();

        // Try to update, map empty changeset to None (instead of Error)
        let pot_empty_changeset: Option<DbReview> = diesel::update(reviews::table)
            .filter(reviews::id.eq(review_id))
            .set(changeset)
            .get_result::<DbReview>(conn)
            .optional_empty_changeset()
            .expect("Error while updating review");

        // Use non-empty changeset if present and fall back to querying otherwise
        let result = pot_empty_changeset.unwrap_or_else(|| {
            // Fallback query that returns the review as it is stored in the database
            reviews::table
                .filter(reviews::id.eq(review_id))
                .select(DbReview::as_select())
                .first(conn)
                .expect("Unable to get updated review")
        });

        Ok(result.into())
    }
}
