use async_trait::async_trait;
use std::error::Error;

#[async_trait]
pub trait Hasher {
    type Error: Error + Send + Sync + 'static;
    async fn hash_password(password: String) -> Result<String, Self::Error>;
    async fn compare_password(plain: String, hashed: String) -> Result<bool, Self::Error>;
}
