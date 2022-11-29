use crate::ports::Hasher;
use async_trait::async_trait;
use sha_crypt::{sha512_check, sha512_simple, Sha512Params};
use thiserror::Error;

#[derive(Clone)]
pub struct ShaHasher;

#[derive(Error, Debug)]
pub enum ShaHasherError {
    #[error("hash error")]
    HashError,
    #[error("spawn error: {0}")]
    SpawnError(#[from] tokio::task::JoinError),
}

#[async_trait]
impl Hasher for ShaHasher {
    type Error = ShaHasherError;

    async fn hash_password(password: String) -> Result<String, Self::Error> {
        tokio::task::spawn_blocking(move || {
            let params = Sha512Params::new(10_000).or(Err(ShaHasherError::HashError))?;
            sha512_simple(&password, &params).or(Err(ShaHasherError::HashError))
        })
        .await?
    }

    async fn compare_password(
        plain_password: String,
        hashed_password: String,
    ) -> Result<bool, Self::Error> {
        tokio::task::spawn_blocking(move || {
            Ok(sha512_check(&plain_password, &hashed_password).is_ok())
        })
        .await?
    }
}
