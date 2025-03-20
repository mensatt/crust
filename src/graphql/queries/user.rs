use async_graphql::{Context, Result, SimpleObject};
use diesel::prelude::*;

use crate::db::{conn::DbPool, models::user::DbUser};

#[derive(Debug, SimpleObject)]
#[graphql(name = "User")]
pub struct GqlUser {
    pub id: uuid::Uuid,
    pub email: String,
}

impl From<DbUser> for GqlUser {
    fn from(value: DbUser) -> Self {
        GqlUser {
            id: value.id,
            email: value.email,
        }
    }
}

#[derive(Default)]
pub struct UserQueries;

#[async_graphql::Object]
impl UserQueries {
    async fn users(&self, ctx: &Context<'_>) -> Result<Vec<GqlUser>> {
        use crate::schema::users::dsl::*;

        // Get DB connection
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        // TODO: Querying emails should only be possible with authentification
        let results = users
            .select(DbUser::as_select())
            .load(conn)
            .expect("Error loading users");
        Ok(results.into_iter().map(Into::into).collect())
    }
}
