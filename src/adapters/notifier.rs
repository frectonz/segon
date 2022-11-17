use crate::ports::GameStartNotifier;
use async_trait::async_trait;
use tokio::sync::broadcast::{channel, error::SendError, Sender};

#[derive(Clone)]
pub struct Notifier {
    sender: Sender<()>,
}

impl Notifier {
    pub fn new() -> Self {
        let (sender, _) = channel::<()>(1);
        Self { sender }
    }
}

impl Default for Notifier {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GameStartNotifier for Notifier {
    type Error = SendError<()>;
    async fn wait_for_signal(&self) -> Option<()> {
        let mut rx = self.sender.subscribe();
        rx.recv().await.ok()
    }

    async fn send_signal(&self) -> Result<(), Self::Error> {
        self.sender.send(())?;
        Ok(())
    }
}
