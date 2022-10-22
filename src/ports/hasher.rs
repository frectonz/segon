use async_trait::async_trait;

#[async_trait]
pub trait Hasher {
    async fn hash_password(self, password: String) -> String;
    async fn compare_password(self, plain: String, hashed: String) -> bool;
}
