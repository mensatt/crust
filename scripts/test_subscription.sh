#!/bin/bash
# Test GraphQL Subscriptions via WebSocket

echo "Testing GraphQL Subscription: reviewCreated"
echo "============================================"
echo ""
echo "Connecting to ws://localhost:8000/graphql/ws..."
echo ""

(
  # Send connection init
  echo '{"type":"connection_init"}'
  sleep 0.5

  # Subscribe to reviewCreated
  echo '{"id":"1","type":"start","payload":{"query":"subscription { reviewCreated { id displayName stars text createdAt } }"}}'

  # Keep connection open for 60 seconds to receive events
  sleep 60

  # Stop subscription
  echo '{"id":"1","type":"stop"}'

  # Close connection
  echo '{"type":"connection_terminate"}'
) | websocat --protocol graphql-ws ws://localhost:8000/graphql/ws
