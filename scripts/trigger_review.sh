#!/bin/bash
# Trigger a review creation to test subscriptions

# First, get a valid occurrence UUID from the database
echo "Fetching a valid occurrence UUID..."

OCCURRENCE_UUID=$(curl -s http://localhost:8000/graphql \
  -H "Content-Type: application/json" \
  -d '{"query":"{ occurrences { id } }"}' | \
  grep -o '"id":"[^"]*' | head -1 | cut -d'"' -f4)

if [ -z "$OCCURRENCE_UUID" ]; then
  echo "Error: No occurrences found in database"
  echo "Creating a test occurrence first might be needed"
  exit 1
fi

echo "Using occurrence: $OCCURRENCE_UUID"
echo ""
echo "Creating review..."
echo ""

curl -X POST http://localhost:8000/graphql \
  -H "Content-Type: application/json" \
  -d "{\"query\":\"mutation { createReview(input: { occurrence: \\\"$OCCURRENCE_UUID\\\", displayName: \\\"Test User\\\", stars: 5, text: \\\"Testing subscriptions!\\\" }) { id displayName stars text createdAt } }\"}" | jq

echo ""
echo "Review created! Check your subscription listener for the event."
