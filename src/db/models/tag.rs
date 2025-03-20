use diesel::prelude::*;

#[derive(Debug, Queryable, Selectable, Insertable, Identifiable)]
#[diesel(primary_key(key))]
#[diesel(table_name = crate::schema::tags)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbTag {
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
