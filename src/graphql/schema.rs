use crate::graphql::{mutations::*, queries::*};

#[derive(async_graphql::MergedObject, Default)]
pub struct Query(
    DishQueries,
    LocationQueries,
    OccurrenceQueries,
    ReviewQueries,
    TagQueries,
    UserQueries,
);

#[derive(async_graphql::MergedObject, Default)]
pub struct Mutation(
    DishMutations,
    LocationMutations,
    OccurrenceMutations,
    ReviewMutations,
    TagMutations,
    UserMutations,
);
