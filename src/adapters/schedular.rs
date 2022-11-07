use crate::ports::{GameStartNotifier, JobSchedular};
use async_trait::async_trait;
use std::time::Duration;
use tokio_cron_scheduler::Job;

#[derive(Clone)]
pub struct Schedular {
    schedular: tokio_cron_scheduler::JobScheduler,
}

impl Schedular {
    pub async fn new<N: GameStartNotifier + Clone + Send + Sync + 'static>(notifier: &N) -> Self {
        let schedular = tokio_cron_scheduler::JobScheduler::new().await.unwrap();

        let notifier = notifier.clone();
        let game_start_job = Job::new_async("1/10 * * * * *", move |_uuid, _l| {
            let notifier = notifier.clone();
            Box::pin(async move {
                notifier.send_signal().await;
            })
        })
        .unwrap();

        schedular.add(game_start_job).await.unwrap();

        schedular.start().await.unwrap();

        Schedular { schedular }
    }
}

#[async_trait]
impl JobSchedular for Schedular {
    async fn time_till_game(&mut self) -> Duration {
        self.schedular.time_till_next_job().await.unwrap().unwrap()
    }
}
