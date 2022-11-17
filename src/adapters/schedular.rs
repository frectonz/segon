use crate::ports::{GameStartNotifier, JobSchedular};
use async_trait::async_trait;
use std::time::Duration;
use thiserror::Error;
use tokio_cron_scheduler::Job;

#[derive(Clone)]
pub struct Schedular {
    schedular: tokio_cron_scheduler::JobScheduler,
    job_id: uuid::Uuid,
}

#[derive(Debug, Error)]
pub enum SchedularError {
    #[error("error while scheduling job: {0}")]
    JobSchedularError(#[from] tokio_cron_scheduler::JobSchedulerError),
    #[error("couldn't get time till game")]
    CouldNotGetTimeTillGame,
    #[error("couldn't get system time: {0}")]
    CouldNotGetSystemTime(#[from] std::time::SystemTimeError),
}

impl Schedular {
    pub async fn new<N: GameStartNotifier + Clone + Send + Sync + 'static>(
        notifier: N,
    ) -> Result<Self, tokio_cron_scheduler::JobSchedulerError> {
        let schedular = tokio_cron_scheduler::JobScheduler::new().await?;

        let game_start_job = Job::new_async("1/30 * * * * *", move |_, _| {
            let notifier = notifier.clone();
            Box::pin(notifier_callback(notifier))
        })?;

        let job_id = game_start_job.guid();
        schedular.add(game_start_job).await?;
        schedular.start().await?;

        Ok(Schedular { schedular, job_id })
    }
}

async fn notifier_callback<N: GameStartNotifier + Clone + Send + Sync + 'static>(notifier: N) {
    match notifier.send_signal().await {
        Ok(()) => {
            log::info!("game start signal sent");
        }
        Err(e) => {
            log::error!("error while sending game start signal: {}", e);
        }
    };
}

#[async_trait]
impl JobSchedular for Schedular {
    type Error = SchedularError;

    async fn time_till_game(&mut self) -> Result<Duration, Self::Error> {
        let t = self
            .schedular
            .next_tick_for_job(self.job_id)
            .await?
            .ok_or(SchedularError::CouldNotGetTimeTillGame)?
            .timestamp();
        let t = Duration::from_secs(t as u64);
        let n = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?;
        Ok(t - n)
    }
}
