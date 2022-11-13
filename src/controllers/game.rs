use crate::{
    models::{ClientMessage, ServerMessage, User},
    ports::{GameDatabase, GameStartNotifier, JobSchedular},
};
use futures_util::{future, Sink, Stream, StreamExt, TryStreamExt};
use std::{sync::Arc, time::Duration};
use tokio::sync::{mpsc::unbounded_channel, Mutex};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::Message;

#[derive(Clone)]
pub struct GameController<GD, JS, GSN>
where
    GD: GameDatabase,
    JS: JobSchedular,
    GSN: GameStartNotifier,
{
    db: GD,
    schedular: JS,
    notifier: GSN,
}

impl<GD, JS, GSN> GameController<GD, JS, GSN>
where
    GD: GameDatabase + Send + Sync + Clone + 'static,
    JS: JobSchedular + Send + Sync + Clone + 'static,
    GSN: GameStartNotifier + Send + Sync + Clone + 'static,
{
    pub fn new(db: GD, schedular: JS, notifier: GSN) -> Self {
        Self {
            db,
            schedular,
            notifier,
        }
    }

    pub async fn start<Socket>(mut self, user: User, ws: Socket)
    where
        Socket: Stream<Item = Result<Message, warp::Error>> + Sink<Message>,
    {
        let (outgoing, incoming) = ws.split();
        let (tx, rx) = unbounded_channel();
        let rx = UnboundedReceiverStream::new(rx);

        let time = self.schedular.time_till_game().await;
        let message = ServerMessage::TimeTillGame {
            time: time.as_secs(),
        };
        let message = serde_json::to_string(&message).unwrap();
        tx.send(Message::text(message)).unwrap();

        let current_question: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

        let this = self.clone();
        let receive_from_client = incoming.try_for_each(|msg: Message| {
            let msg = msg.to_str().map(serde_json::from_str::<ClientMessage>);

            match msg {
                Ok(Ok(client_msg)) => match client_msg {
                    ClientMessage::Answer { answer_idx: answer } => {
                        let this = this.clone();
                        let username = user.username.clone();
                        let current_question = current_question.clone();
                        let tx = tx.clone();
                        tokio::spawn(async move {
                            match &*current_question.lock().await {
                                Some(question) => {
                                    let _ = this.db.set_answer(&username, question, answer).await;
                                }
                                None => {
                                    tx.send(Message::text(
                                        serde_json::to_string(&ServerMessage::NoGame).unwrap(),
                                    ))
                                    .unwrap();
                                }
                            };
                        });
                    }
                },
                _ => {}
            };

            future::ready(Ok(()))
        });

        let send_to_client = rx.map(Ok).forward(outgoing);

        let username = user.username.clone();
        let current_question = current_question.clone();
        let tx = tx.clone();
        let wait_for_game_to_start = tokio::spawn(async move {
            let tx = Arc::new(Mutex::new(tx));
            while let Some(()) = self.notifier.wait_for_signal().await {
                let game = self.db.get_game().await;

                for (i, question) in game.questions.into_iter().enumerate() {
                    let tx = tx.lock().await;

                    let message = serde_json::to_string(&ServerMessage::Question {
                        question: question.question.clone(),
                        options: question.options,
                    })
                    .unwrap();
                    tx.send(Message::text(message)).unwrap();
                    *current_question.lock().await = Some(question.question.clone());

                    let sleep_time: u64 = (i + 1) as u64 * 10;
                    tokio::time::sleep(Duration::from_secs(sleep_time)).await;
                    let answer = match self.db.get_answer(&username, &question.question).await {
                        Some(answer) => {
                            if answer == question.answer_idx {
                                "Correct"
                            } else {
                                "Incorrect"
                            }
                        }
                        None => "NoAnswer",
                    };

                    let message = serde_json::to_string(&ServerMessage::Answer {
                        status: answer.into(),
                        answer_idx: question.answer_idx,
                    })
                    .unwrap();

                    tx.send(Message::text(message)).unwrap();
                }
            }
        });

        tokio::select! {
            _ = receive_from_client => {}
            _ = wait_for_game_to_start => {}
            _ = send_to_client => {},
        }
    }
}

// add them to a list of connected user -> done
// tell the client how much time is left until the game starts -> done
// when the timer reaches the game time send a start signal and the first question to the client --> done
// receive answers for 10 seconds until the question timer ends --> done
// send the answer to all of the connected clients and whether or not they were correct --> done
// drop off the clients that have answered wrong from the answerers list
// repeat this process until all questions have been answered --> done
// assemble a leaderboard and send it to every client with each client's rank on the leaderboard
