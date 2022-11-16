use async_trait::async_trait;

#[async_trait]
pub trait IDGenerator {
    async fn generate() -> String;
}
