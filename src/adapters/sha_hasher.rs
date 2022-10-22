use crate::ports::Hasher;
use async_trait::async_trait;
use sha_crypt::{sha512_check, sha512_simple, Sha512Params};

#[derive(Default)]
pub struct ShaHasher;

#[async_trait]
impl Hasher for ShaHasher {
    async fn hash_password(self, password: String) -> String {
        let handle = tokio::spawn(async move {
            let params = Sha512Params::new(10_000).expect("RandomError!");
            sha512_simple(&password, &params).expect("Should not fail")
        });

        handle
            .await
            .expect("Failed to `join` async handle SHA hasher")
    }

    async fn compare_password(self, plain: String, password: String) -> bool {
        let handle = tokio::spawn(async move { sha512_check(&plain, &password).is_ok() });

        handle
            .await
            .expect("Failed to `join` async handle SHA hasher")
    }
}
