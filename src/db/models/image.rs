use diesel::prelude::*;

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable)]
#[diesel(belongs_to(DbReview, foreign_key = review))]
#[diesel(table_name = crate::schema::images)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbImage {
    pub id: uuid::Uuid,
    pub review: uuid::Uuid,
}
