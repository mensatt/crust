use async_graphql::{Context, Result, Subscription};
use futures_util::Stream;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

use crate::graphql::error::GqlApiError;
use crate::graphql::queries::GqlReview;
use crate::graphql::subscriptions::broker::{ReviewEvent, SubscriptionBroker};

#[derive(Default)]
pub struct ReviewSubscriptions;

#[Subscription]
impl ReviewSubscriptions {
    /// Subscribe to newly created reviews
    async fn review_created(&self, ctx: &Context<'_>) -> Result<impl Stream<Item = GqlReview>> {
        // Get the subscription broker from context
        let broker = ctx.data::<SubscriptionBroker>().map_err(|e| {
            GqlApiError::internal("Unable to get SubscriptionBroker from context", e.message)
        })?;

        // Subscribe to review events
        let receiver = broker.subscribe_reviews();

        // Convert broadcast receiver to a stream, filter for Created events only
        let stream = BroadcastStream::new(receiver)
            .filter_map(|result| match result {
                Ok(ReviewEvent::Created(review)) => Some(review),
                _ => None,
            });

        Ok(stream)
    }

    /// Subscribe to accepted reviews
    async fn review_accepted(&self, ctx: &Context<'_>) -> Result<impl Stream<Item = GqlReview>> {
        // Get the subscription broker from context
        let broker = ctx.data::<SubscriptionBroker>().map_err(|e| {
            GqlApiError::internal("Unable to get SubscriptionBroker from context", e.message)
        })?;

        // Subscribe to review events
        let receiver = broker.subscribe_reviews();

        // Convert broadcast receiver to a stream, filter for Accepted events only
        let stream = BroadcastStream::new(receiver)
            .filter_map(|result| match result {
                Ok(ReviewEvent::Accepted(review)) => Some(review),
                _ => None,
            });

        Ok(stream)
    }
}
