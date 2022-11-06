use crate::{
    models::{Client, Tx},
    ports::ClientsManager,
};
use async_trait::async_trait;
use std::{
    collections::HashMap,
    sync::{atomic::AtomicUsize, Arc, Mutex},
};

static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone, Default)]
pub struct PeerMap {
    clients: Arc<Mutex<HashMap<usize, Tx>>>,
}

#[async_trait]
impl ClientsManager for PeerMap {
    type Error = String;

    async fn add_client(&self, tx: Tx) -> Result<(), Self::Error> {
        let id = NEXT_USER_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let mut client = self.clients.lock().unwrap();
        client.insert(id, tx);
        Ok(())
    }

    async fn get_clients(&self) -> Vec<Client> {
        let clients = self.clients.lock().unwrap();
        clients
            .iter()
            .map(|(id, tx)| Client {
                id: *id,
                tx: tx.clone(),
            })
            .collect()
    }

    async fn remove_client(&self, id: usize) -> () {
        let mut clients = self.clients.lock().unwrap();
        clients.remove(&id);
    }
}
