use crate::ports::GameStartNotifier;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::{mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender}, Mutex};

#[derive(Clone)]
pub struct Notifier {
    reciever: Arc<Mutex<UnboundedReceiver<()>>>,
    sender: UnboundedSender<()>
}

impl Notifier {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded_channel::<()>();
        Self { 
            reciever: Arc::new(Mutex::new(receiver)), 
            sender }
    }
}

#[async_trait]
impl GameStartNotifier for Notifier {
    async fn wait_for_signal(&self) -> Option<()> {
        let mut rx = self.reciever.lock().await;
        rx.recv().await
    }

    async fn send_signal(&self) -> () {
        self.sender.send(()).unwrap()
    }
}
