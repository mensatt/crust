use async_graphql::Schema;

use crate::graphql::{mutations::*, queries::*, subscriptions::*};

#[derive(async_graphql::MergedObject, Default)]
pub struct Query(
    DishQueries,
    ImageQueries,
    LocationQueries,
    OccurrenceQueries,
    ReviewQueries,
    TagQueries,
    UserQueries,
);

#[derive(async_graphql::MergedObject, Default)]
pub struct Mutation(
    DishMutations,
    DishAliasMutations,
    LocationMutations,
    OccurrenceMutations,
    ReviewMutations,
    TagMutations,
    UserMutations,
);

#[derive(async_graphql::MergedSubscription, Default)]
pub struct Subscription(ReviewSubscriptions);

// Small helper type to avoid code duplication
pub type GqlSchema = Schema<Query, Mutation, Subscription>;
