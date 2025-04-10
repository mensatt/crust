// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "tag_priority"))]
    pub struct TagPriority;
}

diesel::table! {
    dishes (id) {
        id -> Uuid,
        name_de -> Varchar,
        name_en -> Nullable<Varchar>,
    }
}

diesel::table! {
    locations (id) {
        id -> Uuid,
        external_id -> Int8,
        name -> Varchar,
        visible -> Bool,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::TagPriority;

    tags (key) {
        key -> Varchar,
        name -> Varchar,
        description -> Varchar,
        short_name -> Nullable<Varchar>,
        priority -> TagPriority,
        is_allergy -> Bool,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        email -> Varchar,
        password_hash -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::allow_tables_to_appear_in_same_query!(dishes, locations, tags, users,);
