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
    occurrences (id) {
        id -> Uuid,
        date -> Timestamptz,
        kj -> Nullable<Int8>,
        kcal -> Nullable<Int8>,
        fat -> Nullable<Int8>,
        saturated_fat -> Nullable<Int8>,
        carbohydrates -> Nullable<Int8>,
        sugar -> Nullable<Int8>,
        fiber -> Nullable<Int8>,
        protein -> Nullable<Int8>,
        salt -> Nullable<Int8>,
        price_student -> Nullable<Int8>,
        price_staff -> Nullable<Int8>,
        price_guest -> Nullable<Int8>,
        dish -> Uuid,
        location -> Uuid,
        not_available_after -> Nullable<Timestamptz>,
        status -> Varchar,
    }
}

diesel::table! {
    occurrences_side_dishes (occurrence, dish) {
        occurrence -> Uuid,
        dish -> Uuid,
    }
}

diesel::table! {
    occurrences_tags (occurrence, tag) {
        occurrence -> Uuid,
        tag -> Varchar,
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

diesel::joinable!(occurrences -> dishes (dish));
diesel::joinable!(occurrences -> locations (location));
diesel::joinable!(occurrences_side_dishes -> dishes (dish));
diesel::joinable!(occurrences_side_dishes -> occurrences (occurrence));
diesel::joinable!(occurrences_tags -> occurrences (occurrence));
diesel::joinable!(occurrences_tags -> tags (tag));

diesel::allow_tables_to_appear_in_same_query!(
    dishes,
    locations,
    occurrences,
    occurrences_side_dishes,
    occurrences_tags,
    tags,
    users,
);
