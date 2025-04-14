use diesel::prelude::*;

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable)]
#[diesel(table_name = crate::schema::locations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbLocation {
    pub id: uuid::Uuid,
    pub external_id: i64,
    pub name: String,
    pub visible: bool,
}
