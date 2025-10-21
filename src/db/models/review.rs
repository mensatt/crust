use crate::db::models::occurrence::DbOccurrence;
use diesel::prelude::*;

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, Associations)]
#[diesel(belongs_to(DbOccurrence, foreign_key = occurrence))]
#[diesel(table_name = crate::schema::reviews)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbReview {
    pub id: uuid::Uuid,
    pub display_name: Option<String>,
    pub stars: i64,
    pub text: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub accepted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub occurrence: uuid::Uuid,
}

#[derive(Debug, Queryable, AsChangeset)]
#[diesel(table_name = crate::schema::reviews)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbReviewChangeset {
    pub display_name: Option<String>,
    pub stars: Option<i64>,
    pub text: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub accepted_at: Option<Option<chrono::DateTime<chrono::Utc>>>,
    pub occurrence: Option<uuid::Uuid>,
}
