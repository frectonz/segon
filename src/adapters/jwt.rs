use crate::ports::{Claims, TokenGenerator};
use async_trait::async_trait;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

const SECRET: &str = "secret";
const EXPIRATION: u64 = 24 * 60 * 60;

#[derive(Clone)]
pub struct Jwt;

#[derive(Error, Debug)]
pub enum JwtError {
    #[error("jwt error: {0}")]
    TokenError(#[from] jsonwebtoken::errors::Error),
    #[error("time error: {0}")]
    TimeError(#[from] std::time::SystemTimeError),
    #[error("spawn error: {0}")]
    SpawnError(#[from] tokio::task::JoinError),
}

#[async_trait]
impl TokenGenerator for Jwt {
    type Error = JwtError;
    async fn generate(id: String) -> Result<String, Self::Error> {
        tokio::task::spawn_blocking(|| {
            let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
            let claims = Claims::new(EXPIRATION, now.as_secs(), id.into());

            let token = encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(SECRET.as_ref()),
            )?;

            Ok(token)
        })
        .await?
    }

    async fn get_claims(token: String) -> Result<Claims, Self::Error> {
        tokio::task::spawn_blocking(move || {
            let mut validation = Validation::default();
            validation.validate_exp = false;

            Ok(decode::<Claims>(
                &token,
                &DecodingKey::from_secret(SECRET.as_ref()),
                &validation,
            )
            .map(|data| data.claims)?)
        })
        .await?
    }
}
