use std::sync::Arc;
use tokio::sync::broadcast;

use crate::graphql::subscriptions::review::ReviewEvent;

/// Broker for managing subscription events
/// Uses Tokio's broadcast channel for efficient fan-out messaging
#[derive(Clone)]
pub struct SubscriptionBroker {
    /// Broadcast channel for review events
    review_tx: Arc<broadcast::Sender<ReviewEvent>>,
}

impl SubscriptionBroker {
    /// Create a new subscription broker
    pub fn new() -> Self {
        // Capacity of 1024 should be sufficient for most cases
        // Each receiver has its own buffer - if a slow receiver falls behind by more than
        // 1024 messages, it will receive a RecvError::Lagged and the oldest messages are dropped
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
