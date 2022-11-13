use crate::ports::{Claims, TokenGenerator};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

const SECRET: &str = "secret";

#[derive(Clone)]
pub struct Jwt;

#[derive(Error, Debug)]
pub enum JwtError {
    #[error("invalid token")]
    TokenError(#[from] jsonwebtoken::errors::Error),
}

impl TokenGenerator for Jwt {
    type Error = JwtError;
    fn generate(&self, username: &str) -> Result<String, Self::Error> {
        let claims = Claims {
            exp: 24 * 60 * 60,
            iat: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            sub: username.into(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(SECRET.as_ref()),
        )?;

        Ok(token)
    }

    fn get_claims(&self, token: String) -> Option<Claims> {
        let mut validation = Validation::default();
        validation.validate_exp = false;

        decode::<Claims>(
            &token,
            &DecodingKey::from_secret(SECRET.as_ref()),
            &validation,
        )
        .ok()
        .map(|data| data.claims)
    }
}
