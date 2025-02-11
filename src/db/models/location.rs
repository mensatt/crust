use async_graphql::{InputObject, SimpleObject};
use diesel::prelude::*;

#[derive(Debug, Queryable, Insertable, Selectable, SimpleObject)]
#[diesel(table_name = crate::schema::locations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Location {
    pub id: uuid::Uuid,
    pub external_id: i64,
    pub name: String,
    pub visible: bool,
}

#[derive(Debug, Queryable, Insertable, InputObject)]
#[diesel(table_name = crate::schema::locations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CreateLocationInput {
    pub external_id: i64,
    pub name: String,
    pub visible: Option<bool>,
}

#[derive(Debug, InputObject, AsChangeset)]
#[diesel(table_name = crate::schema::locations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UpdateLocationInput {
    pub id: uuid::Uuid,
    pub external_id: Option<i64>,
    pub name: Option<String>,
    pub visible: Option<bool>,
}
