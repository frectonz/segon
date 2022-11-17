use async_trait::async_trait;

#[async_trait]
pub trait GameStartNotifier {
    type Error: std::error::Error + Send + Sync + 'static;
    async fn wait_for_signal(&self) -> Option<()>;
    async fn send_signal(&self) -> Result<(), Self::Error>;
}
