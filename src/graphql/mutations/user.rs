use std::sync::Arc;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use async_graphql::{Context, InputObject, Result};
use diesel::prelude::*;

use crate::auth::{create_jwt, Claims, JwtKeyPair};
use crate::db::{conn::DbPool, models::user::DbUser};
use crate::graphql::queries::GqlUser;
use crate::schema::users;

#[derive(Debug, InputObject)]
pub struct CreateUserInput {
    pub email: String,
    pub password: String,
}

#[derive(Debug, InputObject)]
pub struct LoginUserInput {
    pub email: String,
    pub password: String,
}

#[derive(Default)]
pub struct UserMutations;

#[async_graphql::Object]
impl UserMutations {
    async fn login_user(&self, ctx: &Context<'_>, input: LoginUserInput) -> Result<String> {
        // Get DB connection
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        // Check if there is a user with the given email
        let user = users::table
            .filter(users::email.eq(input.email))
            .first::<DbUser>(conn)
            .map_err(|_| async_graphql::Error::new("Invalid email or password"))?;

        // Check given password against hash from DB
        // TODO: Think about if we want to support old bcrypt hashes
        let parsed_hash = PasswordHash::new(&user.password_hash)?;
        let password_ok = Argon2::default()
            .verify_password(input.password.as_bytes(), &parsed_hash)
            .is_ok();
        if !password_ok {
            return Err(async_graphql::Error::new("Invalid email or password"));
        }

        // Extract (reference to) encoding key from context
        let encoding_key = &ctx.data::<Arc<JwtKeyPair>>()?.encoding_key;
        // Create JWT for this user
        // TODO: Error handling
        let jwt = create_jwt(&Claims::new(user.email), encoding_key).unwrap();

        Ok(jwt)
    }

    async fn create_user(&self, ctx: &Context<'_>, input: CreateUserInput) -> Result<GqlUser> {
        // Get DB connection
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        // Generate random salt for password hash
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
