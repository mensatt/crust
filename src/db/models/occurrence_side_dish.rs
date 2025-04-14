use crate::db::models::{dish::DbDish, occurrence::DbOccurrence};
use diesel::prelude::*;

#[derive(Debug, Queryable, Selectable, Insertable, Identifiable, Associations)]
#[diesel(belongs_to(DbOccurrence, foreign_key = occurrence))]
#[diesel(belongs_to(DbDish, foreign_key = dish))]
#[diesel(table_name = crate::schema::occurrences_side_dishes)]
#[diesel(primary_key(occurrence, dish))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbOccurrenceSideDish {
    pub occurrence: uuid::Uuid,
    pub dish: uuid::Uuid,
}
