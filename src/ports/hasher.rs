use async_trait::async_trait;

#[async_trait]
pub trait Hasher {
    async fn hash_password<'a>(&'a self, password: &str) -> String;
    async fn compare_password<'a>(&'a self, plain: &str, hashed: &str) -> bool;
}
