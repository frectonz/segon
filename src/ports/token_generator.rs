use async_trait::async_trait;
use serde::{Deserialize, Serialize};

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
    type Error;
    fn generate(id: &str) -> Result<String, Self::Error>;
    fn get_claims(token: &str) -> Option<Claims>;
}
