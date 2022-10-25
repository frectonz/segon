use async_trait::async_trait;

#[async_trait]
pub trait Hasher {
    async fn hash_password(self, password: &str) -> String;
    async fn compare_password(self, plain: &str, hashed: &str) -> bool;
}
