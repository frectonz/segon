use async_trait::async_trait;

#[async_trait]
pub trait Hasher {
    async fn hash_password(password: &str) -> String;
    async fn compare_password(plain: &str, hashed: &str) -> bool;
}
