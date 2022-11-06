use crate::models::{Client, Tx};
use async_trait::async_trait;

#[async_trait]
pub trait ClientsManager {
    type Error;
    async fn add_client(&self, tx: Tx) -> Result<(), Self::Error>;
    async fn get_clients(&self) -> Vec<Client>;
    async fn remove_client(&self, id: usize);
}
