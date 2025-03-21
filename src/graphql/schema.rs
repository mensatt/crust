use crate::graphql::{mutations::*, queries::*};

#[derive(async_graphql::MergedObject, Default)]
pub struct Query(
    DishQueries,
    LocationQueries,
    OccurrenceQueries,
    TagQueries,
    UserQueries,
);

#[derive(async_graphql::MergedObject, Default)]
pub struct Mutation(
    DishMutations,
    LocationMutations,
    OccurrenceMutations,
    TagMutations,
    UserMutations,
);
