use async_graphql::{InputObject, SimpleObject};
use diesel::prelude::*;

#[derive(Debug, Queryable, Insertable, Selectable, SimpleObject)]
#[diesel(table_name = crate::schema::tags)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Tag {
    pub key: String,
    pub name: String,
    pub description: String,
    pub short_name: Option<String>,
    pub priority: TagPriority,
    pub is_allergy: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, async_graphql::Enum, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::TagPriority"]
#[DbValueStyle = "UPPERCASE"]
pub enum TagPriority {
    HIGH,
    MEDIUM,
    LOW,
    HIDE,
}

#[derive(Debug, Queryable, Insertable, InputObject)]
#[diesel(table_name = crate::schema::tags)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CreateTagInput {
    pub key: String,
    pub name: String,
    pub description: String,
    pub short_name: Option<String>,
    pub priority: TagPriority,
    pub is_allergy: Option<bool>,
}

#[derive(Debug, InputObject, AsChangeset)]
#[diesel(table_name = crate::schema::tags)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UpdateTagInput {
    pub key: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub short_name: Option<String>,
    pub priority: Option<TagPriority>,
    pub is_allergy: Option<bool>,
}
