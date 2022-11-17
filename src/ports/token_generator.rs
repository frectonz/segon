use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    exp: u64,
    iat: u64,
    sub: String,
}

impl Claims {
    pub fn new(exp: u64, iat: u64, sub: String) -> Self {
        Self { exp, iat, sub }
    }

    pub fn exp(&self) -> u64 {
        self.exp
    }

    pub fn iat(&self) -> u64 {
        self.iat
    }

    pub fn sub(&self) -> &str {
        &self.sub
    }
}

#[async_trait]
pub trait TokenGenerator {
    type Error: Error + Send + Sync + 'static;
    async fn generate(id: String) -> Result<String, Self::Error>;
    async fn get_claims(token: String) -> Result<Claims, Self::Error>;
}
