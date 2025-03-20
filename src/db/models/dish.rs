use diesel::prelude::*;

#[derive(Debug, Queryable, Selectable, Insertable, Identifiable)]
#[diesel(table_name = crate::schema::dishes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbDish {
    pub id: uuid::Uuid,
    pub name_de: String,
    pub name_en: Option<String>,
}
