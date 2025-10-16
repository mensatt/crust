use std::sync::Arc;
use tokio::sync::broadcast;

use crate::graphql::queries::GqlReview;

/// Events that can be broadcast to subscribers
#[derive(Clone, Debug)]
pub enum ReviewEvent {
    Created(GqlReview),
    Accepted(GqlReview),
}

/// Broker for managing subscription events
/// Uses Tokio's broadcast channel for efficient fan-out messaging
#[derive(Clone)]
pub struct SubscriptionBroker {
    // Broadcast channel for review events
    // We use a capacity of 1024 which should be sufficient for most cases
    // Older messages will be dropped if the channel fills up
    review_tx: Arc<broadcast::Sender<ReviewEvent>>,
}

impl SubscriptionBroker {
    /// Create a new subscription broker
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self {
            review_tx: Arc::new(tx),
        }
    }

    /// Publish a review event to all subscribers
    pub fn publish_review(&self, event: ReviewEvent) {
        // We don't care if there are no receivers, so ignore errors
        let _ = self.review_tx.send(event);
    }

    /// Subscribe to review events
    pub fn subscribe_reviews(&self) -> broadcast::Receiver<ReviewEvent> {
        self.review_tx.subscribe()
    }
}

impl Default for SubscriptionBroker {
    fn default() -> Self {
        Self::new()
    }
}
