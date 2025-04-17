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
    LocationMutations,
    OccurrenceMutations,
    ReviewMutations,
    TagMutations,
    UserMutations,
);
