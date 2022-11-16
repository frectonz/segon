use crate::ports::Hasher;
use async_trait::async_trait;
use sha_crypt::{sha512_check, sha512_simple, Sha512Params};

#[derive(Clone)]
pub struct ShaHasher;

#[async_trait]
impl Hasher for ShaHasher {
    async fn hash_password(password: &str) -> String {
        let params = Sha512Params::new(10_000).expect("RandomError!");
        sha512_simple(password, &params).expect("Should not fail")
    }

    async fn compare_password(plain_password: &str, hashed_password: &str) -> bool {
        sha512_check(plain_password, hashed_password).is_ok()
    }
}
