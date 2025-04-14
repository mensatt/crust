use diesel::prelude::*;

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable)]
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
#[db_enum(existing_type_path = "crate::schema::sql_types::TagPriority")]
#[db_enum(value_style = "UPPERCASE")]
pub enum TagPriority {
    HIGH,
    MEDIUM,
    LOW,
    HIDE,
}
