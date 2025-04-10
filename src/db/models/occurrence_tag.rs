use crate::db::models::{occurrence::DbOccurrence, tag::DbTag};
use diesel::prelude::*;

#[derive(Debug, Queryable, Selectable, Insertable, Identifiable, Associations)]
#[diesel(belongs_to(DbOccurrence, foreign_key = occurrence))]
#[diesel(belongs_to(DbTag, foreign_key = tag))]
#[diesel(table_name = crate::schema::occurrences_tags)]
#[diesel(primary_key(occurrence, tag))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbOccurrenceTag {
    pub occurrence: uuid::Uuid,
    pub tag: String,
}
