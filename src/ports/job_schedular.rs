use async_trait::async_trait;
use std::time::Duration;

#[async_trait]
pub trait JobSchedular {
    async fn time_till_game(&mut self) -> Duration;
}
