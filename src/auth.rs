use std::fs;

use anyhow::Context;
use chrono::{Duration, Utc};
use jsonwebtoken::errors::Result;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use log::info;
use serde::{Deserialize, Serialize};

use crate::config::JwtConfig;

// NOTE: We don't include this in JwtConfig because changing the algorithm would probably
//       require code changes anyway.
//       For example, switching from RS256 to HS256 means we'd need to use the same key for
//       both signing and verifying.
pub const JWT_ALGO: Algorithm = Algorithm::RS256;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Subject (we use email)
    pub iat: i64,    // Issued at timestamp (in s since epoch)
    pub nbf: i64,    // Not before timestamp (in s since epoch)
    pub exp: i64,    // Expiration of the token (in s since epoch)
}

impl Claims {
    pub fn new(user_id: String, lifetime_in_secs: i64) -> Self {
        let now = Utc::now();
        let iat = now.timestamp();
        let nbf = now.timestamp();
        // Token shall be valid for `lifetime_in_secs` seconds from signing
        let exp = now
            .checked_add_signed(Duration::seconds(lifetime_in_secs))
            .expect("Unable to compute claim expiry date")
            .timestamp();

        Claims {
            sub: user_id,
            iat,
            nbf,
            exp,
        }
    }
}

pub struct JwtKeyPair {
    pub encoding_key: EncodingKey, // The private key used to create signatures
    pub decoding_key: DecodingKey, // The public  key used to verify signatures
}

pub fn init_jwt_keypair(jwt_config: &JwtConfig) -> anyhow::Result<JwtKeyPair> {
    let private_key = fs::read(&jwt_config.private_key_path).with_context(|| {
        format!(
            "Failed to read private key from {}",
            jwt_config.private_key_path
        )
    })?;

    let public_key = fs::read(&jwt_config.public_key_path).with_context(|| {
        format!(
            "Failed to read public key from {}",
            jwt_config.public_key_path
        )
    })?;

    let encoding_key = EncodingKey::from_rsa_pem(&private_key)
        .context("Failed to create encoding key from private key")?;

    let decoding_key = DecodingKey::from_rsa_pem(&public_key)
        .context("Failed to create decoding key from public key")?;

    info!("Successfully created JWT keypair");

    Ok(JwtKeyPair {
        encoding_key,
        decoding_key,
    })
}

pub fn create_jwt(claims: &Claims, encoding_key: &EncodingKey) -> Result<String> {
    encode(&Header::new(JWT_ALGO), claims, encoding_key)
}

pub fn verify_jwt(token: &str, decoding_key: &DecodingKey) -> Result<Claims> {
    decode::<Claims>(token, decoding_key, &Validation::new(JWT_ALGO))
        .map(|token_data| token_data.claims)
}

// This will later be passed to GraphQL resolvers via the schema context
pub struct AuthContext {
    pub claims: Option<Claims>,
}

impl AuthContext {
    // Helper function to ensure authorized access in Gql resolvers
    // TODO: It was tried to implement a directive for this, but issues were encountered
    //       For details see: https://github.com/async-graphql/async-graphql/issues/1710
    pub fn require_auth(&self) -> async_graphql::Result<&Claims> {
        self.claims
            .as_ref()
            .ok_or_else(|| async_graphql::Error::new("Unauthorized"))
    }
}
