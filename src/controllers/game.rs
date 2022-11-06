use crate::ports::{ClientsManager, GameDatabase, GameStartNotifier, JobSchedular};
use futures_util::{StreamExt, TryStreamExt};
use std::time::Duration;
use tokio::sync::mpsc::unbounded_channel;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};

#[derive(Clone)]
pub struct GameController<GD, CM, JS, GSN>
where
    GD: GameDatabase,
    CM: ClientsManager,
    JS: JobSchedular,
    GSN: GameStartNotifier,
{
    db: GD,
    clients: CM,
    schedular: JS,
    notifier: GSN,
}

impl<GD, CM, JS, GSN> GameController<GD, CM, JS, GSN>
where
    GD: GameDatabase + Send + Sync + Clone + 'static,
    CM: ClientsManager + Send + Sync + Clone + 'static,
    JS: JobSchedular + Send + Sync + Clone + 'static,
    GSN: GameStartNotifier + Send + Sync + Clone + 'static,
{
    pub fn new(db: GD, clients: CM, schedular: JS, notifier: GSN) -> Self {
        Self {
            db,
            clients,
            schedular,
            notifier,
        }
    }

    pub async fn start(mut self, ws: WebSocket) {
        let (outgoing, incoming) = ws.split();
        let (tx, rx) = unbounded_channel();

        let time = self.schedular.time_till_game().await;
        let message = format!("{}s till game starts", time.as_secs());

        tx.send(Message::text(message)).unwrap();

        let _ = self.clients.add_client(tx);

        let rx = UnboundedReceiverStream::new(rx);

        let receive_from_client = incoming.try_for_each(|msg| async move {
            println!("received msg: {}", msg.to_str().unwrap());
            Ok(())
        });

        let send_to_client = rx.map(Ok).forward(outgoing);

        let this = self.clone();
        let wait_for_game_to_start = tokio::spawn(async move {
            while let Some(()) = self.notifier.wait_for_signal().await {
                let game = self.db.get_game().await;

                for (i, question) in game.questions.into_iter().enumerate() {
                    let this = this.clone();
                    tokio::spawn(async move {
                        let sleep_time: u64 = i as u64 * 10;
                        let clients = this.clients.get_clients().await;
                        tokio::time::sleep(Duration::from_secs(sleep_time)).await;

                        for client in clients.iter() {
                            client.tx.send(Message::text(&question.question)).unwrap();
                        }
                    });
                }
            }
        });

        // pin_mut!(receive_from_client, send_to_client);
        tokio::select! {
            _ = receive_from_client => {}
            // _ = wait_for_game_to_start => {}
            _ = send_to_client => {},
        }
    }
}
