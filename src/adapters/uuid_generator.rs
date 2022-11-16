use crate::ports::IDGenerator;
use async_trait::async_trait;

#[derive(Clone)]
pub struct UuidGenerator;

#[async_trait]
impl IDGenerator for UuidGenerator {
    async fn generate() -> String {
        uuid::Uuid::new_v4().to_string()
    }
}
