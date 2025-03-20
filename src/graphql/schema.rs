use crate::graphql::{mutations::*, queries::*};

#[derive(async_graphql::MergedObject, Default)]
pub struct Query(DishQueries, LocationQueries, TagQueries, UserQueries);

#[derive(async_graphql::MergedObject, Default)]
pub struct Mutation(
    DishMutations,
    LocationMutations,
    TagMutations,
    UserMutations,
);
