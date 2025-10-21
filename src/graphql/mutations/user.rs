use std::sync::Arc;

use argon2::{
    password_hash::{
        errors::Error as PasswordHashError, PasswordHasher, SaltString,
    },
    Argon2, PasswordHash, PasswordVerifier,
};
use async_graphql::{Context, InputObject, Result};
use diesel::{prelude::*, result::Error as DieselError};
use log::info;
// See if this import from rand_core can be removed once
//   https://github.com/RustCrypto/password-hashes/issues/730
// is closed
use rand_core::OsRng;

use crate::db::models::user::DbUser;
use crate::graphql::error::GqlApiError;
use crate::graphql::queries::GqlUser;
use crate::graphql::util::get_conn_from_ctx;
use crate::schema::users;
use crate::{
    auth::{create_jwt, AuthContext, Claims, JwtKeyPair},
    config::AppConfig,
};

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
        let conn = &mut get_conn_from_ctx(ctx)?;

        // Check if there is a user with the given email
        let user = users::table
            .filter(users::email.eq(&input.email))
            .first::<DbUser>(conn)
            .map_err(|e| match e {
                DieselError::NotFound => GqlApiError::InvalidCredentials,
                _ => GqlApiError::internal(
                    format!("Error while fetching user '{}'", input.email),
                    e.to_string(),
                ),
            })?;

        // Parse hash from DB and check if given password matches it
        // TODO: Think about if we want to support old bcrypt hashes
        let parsed_hash = PasswordHash::new(&user.password_hash).map_err(|e| {
            GqlApiError::internal(
                format!(
                    "Unable to read password hash '{}' of user '{}'",
                    user.password_hash, user.email
                ),
                e.to_string(),
            )
        })?;
        Argon2::default()
            .verify_password(input.password.as_bytes(), &parsed_hash)
            // Error handling
            .map_err(|e| match e {
                PasswordHashError::Password => GqlApiError::InvalidCredentials,
                _ => GqlApiError::internal(
                    format!("Unable to verify password for user '{}'", input.email),
                    e.to_string(),
                ),
            })?;

        // Extract JWT lifetime and (reference to) encoding key from context
        let jwt_lifetime = ctx
            .data::<AppConfig>()
            .map_err(|e| GqlApiError::internal("Unable to get AppConfig from context", e.message))?
            .jwt
            .lifetime_in_secs;
        let encoding_key = &ctx
            .data::<Arc<JwtKeyPair>>()
            .map_err(|e| GqlApiError::internal("Unable to get JwtKeyPair from context", e.message))?
            .encoding_key;

        // Create JWT for this user
        let jwt = create_jwt(&Claims::new(user.email, jwt_lifetime), encoding_key)
            .map_err(|e| GqlApiError::internal("Unable to create JWT", e.to_string()))?;

        info!("User '{}' logged in successfully", input.email);

        Ok(jwt)
    }

    async fn create_user(&self, ctx: &Context<'_>, input: CreateUserInput) -> Result<GqlUser> {
        // Require authentication for this mutation
        // TODO: Do we want to allow users to register themselves?
        ctx.data::<AuthContext>()?.require_auth()?;

        // Get DB connection
        let conn = &mut get_conn_from_ctx(ctx)?;

        // Generate random salt for password hash
        let salt = SaltString::generate(&mut OsRng);

        // Argon2 with default params (Argon2id v19)
        let argon2 = Argon2::default();

        // Hash password to PHC string ($argon2id$v=19$...)
        let password_hash = argon2
            .hash_password(input.password.as_bytes(), &salt)
            .map_err(|e| {
                GqlApiError::internal("Unable to create hash for new user", e.to_string())
            })?
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
            // NOTE: In theory .get_result() could return NotFound, but if that happens on insert
            //       something internally has gone wrong.
            .map_err(|e| GqlApiError::internal("Error while inserting new user", e.to_string()))?;

        Ok(results.into())
    }
}
