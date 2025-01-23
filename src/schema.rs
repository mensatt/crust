// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Uuid,
        email -> Varchar,
        password_hash -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}
