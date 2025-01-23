use crate::db::conn::DbPool;
use crate::db::models::user::*;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use async_graphql::Context;
use diesel::prelude::*;

#[derive(Default)]
pub struct QueryRoot;

#[async_graphql::Object]
impl QueryRoot {
    async fn users(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<User>> {
        use crate::schema::users::dsl::*;

        // Get DB connection
        let pool = ctx.data::<DbPool>()?;
        let connection = &mut pool.get().unwrap();

        let results = users
            .select(User::as_select())
            .load(&mut *connection)
            .expect("Error loading users");
        Ok(results)
    }
}

#[derive(Default)]
pub struct MutationRoot;

#[async_graphql::Object]
impl MutationRoot {
    async fn create_user(
        &self,
        ctx: &Context<'_>,
        email: String,
        password: String,
    ) -> async_graphql::Result<User> {
        use crate::schema::users;
        // Get DB connection
        let pool = ctx.data::<DbPool>()?;
        let connection = &mut pool.get().unwrap();

        // Hash password
        let salt = SaltString::generate(&mut OsRng);

        // Argon2 with default params (Argon2id v19)
        let argon2 = Argon2::default();

        // Hash password to PHC string ($argon2id$v=19$...)
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)?
            .to_string();

        let new_user = NewUser {
            id: uuid::Uuid::new_v4(),
            email: &email,
            password_hash: &password_hash,
            created_at: chrono::offset::Utc::now(),
            updated_at: chrono::offset::Utc::now(),
        };
        let results = diesel::insert_into(users::table)
            .values(&new_user)
            .returning(User::as_returning())
            .get_result(connection)
            .expect("Error saving new user");

        Ok(results)
    }
}
