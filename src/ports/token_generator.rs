use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub exp: u64,
    pub iat: u64,
    pub sub: String,
}

#[async_trait]
pub trait TokenGenerator {
    type Error;
    fn generate<'a>(&'a self, username: &str) -> Result<String, Self::Error>;
    fn get_claims<'a>(&'a self, token: String) -> Option<Claims>;
}
