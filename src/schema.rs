// @generated automatically by Diesel CLI.

diesel::table! {
    dishes (id) {
        id -> Uuid,
        name_de -> Varchar,
        name_en -> Nullable<Varchar>,
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

diesel::allow_tables_to_appear_in_same_query!(dishes, users,);
