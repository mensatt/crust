use diesel::prelude::*;

use async_graphql::{InputObject, SimpleObject};

#[derive(Debug, Queryable, Selectable, Insertable, Identifiable, AsChangeset, SimpleObject)]
#[diesel(table_name = crate::schema::dishes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Dish {
    pub id: uuid::Uuid,
    pub name_de: String,
    pub name_en: Option<String>,
}

#[derive(Debug, InputObject)]
pub struct DishFilter {
    pub dishes: Option<Vec<uuid::Uuid>>,
    pub name_de: Option<String>,
    pub name_en: Option<String>,
}

#[derive(Debug, InputObject)]
pub struct CreateDishInput {
    pub name_de: String,
    pub name_en: Option<String>,
}

#[derive(Debug, InputObject, AsChangeset)]
#[diesel(table_name = crate::schema::dishes)]
pub struct UpdateDishInput {
    pub id: uuid::Uuid,
    pub name_de: Option<String>,
    pub name_en: Option<String>,
}
