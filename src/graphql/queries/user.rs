use async_graphql::Context;
use diesel::prelude::*;

use crate::db::{conn::DbPool, models::user::User};

#[derive(Default)]
pub struct UserQueries;

#[async_graphql::Object]
impl UserQueries {
    async fn users(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<User>> {
        use crate::schema::users::dsl::*;

        // Get DB connection
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        let results = users
            .select(User::as_select())
            .load(conn)
            .expect("Error loading users");
        Ok(results)
    }
}
