use crate::{
    models::{AnswerStatus, ClientMessage, ServerMessage},
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

    pub async fn start<Socket>(mut self, user_id: String, ws: Socket)
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

            if let Ok(Ok(client_msg)) = msg {
                match client_msg {
                    ClientMessage::Answer { answer_idx: answer } => {
                        let this = this.clone();
                        let current_question = current_question.clone();
                        let tx = tx.clone();
                        let user_id = user_id.clone();
                        tokio::spawn(async move {
                            match &*current_question.lock().await {
                                Some(question) => {
                                    let _ = this.db.set_answer(&user_id, question, answer).await;
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
                }
            };

            future::ready(Ok(()))
        });

        let send_to_client = rx.map(Ok).forward(outgoing);

        let user_id = user_id.clone();
        let current_question = current_question.clone();
        let tx = tx.clone();
        let wait_for_game_to_start = tokio::spawn(async move {
            let tx = Arc::new(Mutex::new(tx));
            while let Some(()) = self.notifier.wait_for_signal().await {
                let game = self.db.get_game().await;
                let tx = tx.lock().await;

                // send a game start message
                tx.send(Message::text(
                    serde_json::to_string(&ServerMessage::GameStart).unwrap(),
                ))
                .unwrap();

                // wait for 10 seconds
                tokio::time::sleep(Duration::from_secs(10)).await;

                for question in game.questions.into_iter() {
                    // send question
                    let message = serde_json::to_string(&ServerMessage::Question {
                        question: question.question.clone(),
                        options: question.options,
                    })
                    .unwrap();
                    tx.send(Message::text(message)).unwrap();

                    // set current question
                    *current_question.lock().await = Some(question.question.clone());

                    // sleep for 10 seconds
                    tokio::time::sleep(Duration::from_secs(10)).await;

                    // get answer
                    let answer = self.db.get_answer(&user_id, &question.question).await;
                    let answer_status = match answer {
                        Some(answer) => {
                            if answer == question.answer_idx {
                                AnswerStatus::Correct
                            } else {
                                AnswerStatus::Incorrect
                            }
                        }
                        None => AnswerStatus::NoAnswer,
                    };

                    // set answer status
                    let _ = self
                        .db
                        .set_answer_status(&user_id, &question.question, &answer_status)
                        .await;

                    // send answer
                    let message = serde_json::to_string(&ServerMessage::Answer {
                        status: answer_status,
                        answer_idx: question.answer_idx,
                    })
                    .unwrap();
                    tx.send(Message::text(message)).unwrap();

                    // sleep for 10 seconds
                    tokio::time::sleep(Duration::from_secs(10)).await;
                }

                // calculate score
                let score = self
                    .db
                    .get_answers_statuses(&user_id)
                    .await
                    .unwrap_or_default()
                    .iter()
                    .filter(|x| **x == AnswerStatus::Correct)
                    .count() as u32;

                // set score
                let _ = self.db.set_score(&user_id, score).await;

                tx.send(Message::text(
                    serde_json::to_string(&ServerMessage::GameEnd { score }).unwrap(),
                ))
                .unwrap();
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
