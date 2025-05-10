use async_graphql::{EmptySubscription, Schema};

use crate::graphql::{mutations::*, queries::*};

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

// Small helper type to avoid code duplication
pub type GqlSchema = Schema<Query, Mutation, EmptySubscription>;
