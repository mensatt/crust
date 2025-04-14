use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use async_graphql::{Context, InputObject, Result};
use diesel::prelude::*;

use crate::db::{conn::DbPool, models::user::DbUser};
use crate::graphql::queries::GqlUser;
use crate::schema::users;

#[derive(Debug, InputObject)]
pub struct CreateUserInput {
    pub email: String,
    pub password: String,
}

#[derive(Default)]
pub struct UserMutations;

#[async_graphql::Object]
impl UserMutations {
    async fn create_user(&self, ctx: &Context<'_>, input: CreateUserInput) -> Result<GqlUser> {
        // Get DB connection
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        // Hash password
        let salt = SaltString::generate(&mut OsRng);

        // Argon2 with default params (Argon2id v19)
        let argon2 = Argon2::default();

        // Hash password to PHC string ($argon2id$v=19$...)
        let password_hash = argon2
            .hash_password(input.password.as_bytes(), &salt)?
            .to_string();

        let now = chrono::Utc::now();
        let new_user = DbUser {
            id: uuid::Uuid::new_v4(),
            email: input.email,
            password_hash,
            created_at: now,
            updated_at: now,
        };

        // Add user and return it
        let results = diesel::insert_into(users::table)
            .values(&new_user)
            .get_result::<DbUser>(conn)
            .expect("Error saving new user");

        Ok(results.into())
    }
}
