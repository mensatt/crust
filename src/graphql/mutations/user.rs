use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use async_graphql::Context;
use diesel::prelude::*;

use crate::db::{
    conn::DbPool,
    models::user::{NewUser, User},
};

#[derive(Default)]
pub struct UserMutations;

#[async_graphql::Object]
impl UserMutations {
    async fn create_user(
        &self,
        ctx: &Context<'_>,
        email: String,
        password: String,
    ) -> async_graphql::Result<User> {
        use crate::schema::users;
        // Get DB connection
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

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
            .get_result(conn)
            .expect("Error saving new user");

        Ok(results)
    }
}
