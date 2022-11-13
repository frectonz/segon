use crate::ports::GameStartNotifier;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    Mutex,
};

#[derive(Clone)]
pub struct Notifier {
    receiver: Arc<Mutex<UnboundedReceiver<()>>>,
    sender: UnboundedSender<()>,
}

impl Notifier {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded_channel::<()>();
        Self {
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }
}

impl Default for Notifier {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GameStartNotifier for Notifier {
    async fn wait_for_signal(&self) -> Option<()> {
        let mut rx = self.receiver.lock().await;
        rx.recv().await
    }

    async fn send_signal(&self) -> () {
        self.sender.send(()).unwrap()
    }
}
