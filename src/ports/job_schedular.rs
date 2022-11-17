use async_trait::async_trait;
use std::{error::Error, time::Duration};

#[async_trait]
pub trait JobSchedular {
    type Error: Error + Send + Sync + 'static;
    async fn time_till_game(&mut self) -> Result<Duration, Self::Error>;
}
