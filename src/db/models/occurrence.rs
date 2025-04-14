use crate::db::models::{dish::DbDish, location::DbLocation};
use diesel::prelude::*;

#[derive(
    Debug, Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Clone,
)]
#[diesel(belongs_to(DbLocation, foreign_key = location))]
#[diesel(belongs_to(DbDish, foreign_key = dish))]
#[diesel(table_name = crate::schema::occurrences)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbOccurrence {
    pub id: uuid::Uuid,
    pub date: chrono::DateTime<chrono::Utc>,
    pub kj: Option<i64>,
    pub kcal: Option<i64>,
    pub fat: Option<i64>,
    pub saturated_fat: Option<i64>,
    pub carbohydrates: Option<i64>,
    pub sugar: Option<i64>,
    pub fiber: Option<i64>,
    pub protein: Option<i64>,
    pub salt: Option<i64>,
    pub price_student: Option<i64>,
    pub price_staff: Option<i64>,
    pub price_guest: Option<i64>,
    pub dish: uuid::Uuid,
    pub location: uuid::Uuid,
    pub not_available_after: Option<chrono::DateTime<chrono::Utc>>,
    pub status: String,
}

#[derive(Debug, Queryable, AsChangeset)]
#[diesel(table_name = crate::schema::occurrences)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbOccurrenceChangeset {
    pub location: Option<uuid::Uuid>,
    pub dish: Option<uuid::Uuid>,
    pub date: Option<chrono::DateTime<chrono::Utc>>,
    pub kj: Option<i64>,
    pub kcal: Option<i64>,
    pub fat: Option<i64>,
    pub saturated_fat: Option<i64>,
    pub carbohydrates: Option<i64>,
    pub sugar: Option<i64>,
    pub fiber: Option<i64>,
    pub protein: Option<i64>,
    pub salt: Option<i64>,
    pub price_student: Option<i64>,
    pub price_staff: Option<i64>,
    pub price_guest: Option<i64>,
    pub not_available_after: Option<chrono::DateTime<chrono::Utc>>,
    pub status: Option<String>,
}
