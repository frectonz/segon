use async_trait::async_trait;

#[async_trait]
pub trait GameStartNotifier {
    async fn wait_for_signal(&self) -> Option<()>;
    async fn send_signal(&self) -> ();
}
