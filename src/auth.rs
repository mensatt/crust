use std::fs;

use jsonwebtoken::errors::Result;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

pub const JWT_ALGO: Algorithm = Algorithm::RS256;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Subject (we use email)
    pub iat: i64,    // Issued at timestamp (in s since epoch)
    pub nbf: i64,    // Not before timestamp (in s since epoch)
    pub exp: i64,    // Expiration of the token (in s since epoch)
}

impl Claims {
    pub fn new(user_id: String) -> Self {
        let now = chrono::Utc::now();
        let iat = now.timestamp();
        let nbf = now.timestamp();
        // Token shall be valid for 24h from signing
        let duration = chrono::Duration::hours(24);
        let exp = now
            .checked_add_signed(duration)
            .expect("Adding 24h produced invalid timestamp")
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

pub fn init_jwt_keypair() -> JwtKeyPair {
    // TODO: Use config crate for paths
    let private_key = fs::read("private_key.pem").expect("Failed to read private key");
    let public_key = fs::read("public_key.pem").expect("Failed to read public key");

    let encoding_key =
        EncodingKey::from_rsa_pem(&private_key).expect("Failed to create encoding key");
    let decoding_key =
        DecodingKey::from_rsa_pem(&public_key).expect("Failed to create decoding key");

    JwtKeyPair {
        encoding_key,
        decoding_key,
    }
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
