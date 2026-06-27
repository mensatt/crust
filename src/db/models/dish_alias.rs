use crate::db::models::dish::DbDish;
use diesel::prelude::*;

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, Associations)]
#[diesel(belongs_to(DbDish, foreign_key = dish))]
#[diesel(table_name = crate::schema::dishes_aliases)]
#[diesel(primary_key(alias_name))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbDishAlias {
    pub alias_name: String,
    pub normalized_alias_name: String,
    pub dish: uuid::Uuid,
}
