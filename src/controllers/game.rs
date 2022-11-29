use crate::{
    models::{AnswerStatus, ClientMessage, ServerMessage},
    ports::{GameDatabase, GameStartNotifier, JobSchedular},
};
use futures_util::{Sink, Stream, StreamExt, TryStreamExt};
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
        Socket: Stream<Item = Result<Message, warp::Error>> + Sink<Message> + Send + 'static,
    {
        let (outgoing, mut incoming) = ws.split();
        let (tx, rx) = unbounded_channel::<ServerMessage>();
        let rx = UnboundedReceiverStream::new(rx);

        let time = self
            .schedular
            .time_till_game()
            .await
            .map(|time| time.as_secs());

        let time = match time {
            Ok(time) => time,
            Err(e) => {
                log::error!("Failed to get time till game to {user_id}: {e}");
                return;
            }
        };

        match tx.send(ServerMessage::TimeTillGame { time }) {
            Ok(_) => (),
            Err(e) => {
                log::error!("Failed to send time till game to {user_id}: {e}");
                return;
            }
        };

        let current_question: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

        let this_clone = self.clone();
        let current_question_clone = current_question.clone();
        let tx_clone = tx.clone();
        let user_id_clone = user_id.clone();
        let receive_from_client = tokio::spawn(async move {
            while let Ok(msg) = incoming.try_next().await {
                let msg = match msg {
                    Some(msg) => msg,
                    None => {
                        log::error!("Failed to receive message from {user_id_clone}");
                        break;
                    }
                };

                let msg = match msg.to_str() {
                    Ok(msg) => msg,
                    Err(_) => {
                        log::error!("Failed to get message from {user_id_clone}");
                        break;
                    }
                };

                let msg = match serde_json::from_str::<ClientMessage>(msg) {
                    Ok(msg) => msg,
                    Err(e) => {
                        log::error!("Failed to parse message from {user_id_clone}: {e}");
                        break;
                    }
                };

                match msg {
                    ClientMessage::Answer { answer_idx } => {
                        match &*current_question_clone.lock().await {
                            Some(question) => {
                                match this_clone
                                    .db
                                    .set_answer(&user_id_clone, question, answer_idx)
                                    .await
                                {
                                    Ok(()) => (),
                                    Err(e) => {
                                        log::error!(
                                            "Failed to set answer for {user_id_clone}: {e}"
                                        );
                                        continue;
                                    }
                                }
                            }
                            None => {
                                match tx_clone.send(ServerMessage::NoGame) {
                                    Ok(()) => (),
                                    Err(e) => {
                                        log::error!("Failed to send no game message to {user_id_clone}: {e}");
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                };
            }
        });

        let send_to_client = rx
            .map(|msg| {
                let msg = serde_json::to_string(&msg);
                
                match msg {
                    Ok(msg) => Message::text(msg),
                    Err(_) => Message::text("MESSAGE_SERIALIZATION_ERROR"),
                }
            })
            .map(Ok)
            .forward(outgoing);

        let wait_for_game_to_start = tokio::spawn(async move {
            while let Some(()) = self.notifier.wait_for_signal().await {
                let game = self.db.get_game().await;
                let game = match game {
                    Ok(Some(game)) => game,
                    _ => {
                        log::error!("Failed to get game");
                        break;
                    }
                };

                match tx.send(ServerMessage::GameStart) {
                    Ok(()) => (),
                    Err(e) => {
                        log::error!("Failed to send game start message to {user_id}: {e}");
                        break;
                    }
                };

                tokio::time::sleep(Duration::from_secs(10)).await;

                for question in game.questions.into_iter() {
                    match tx.send(ServerMessage::Question {
                        question: question.question.clone(),
                        options: question.options,
                    }) {
                        Ok(()) => (),
                        Err(e) => {
                            log::error!("Failed to send question to {user_id}: {e}");
                            continue;
                        }
                    };

                    *current_question.lock().await = Some(question.question.clone());

                    tokio::time::sleep(Duration::from_secs(10)).await;

                    let answer = self.db.get_answer(&user_id, &question.question).await;
                    let answer = match answer {
                        Ok(answer) => answer,
                        Err(e) => {
                            log::error!("Failed to get answer for {user_id}: {e}");
                            continue;
                        }
                    };

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

                    match self
                        .db
                        .set_answer_status(&user_id, &question.question, &answer_status)
                        .await
                    {
                        Ok(_) => (),
                        Err(e) => {
                            log::error!("Failed to set answer status for {user_id}: {e}");
                            continue;
                        }
                    };

                    match tx.send(ServerMessage::Answer {
                        status: answer_status,
                        answer_idx: question.answer_idx,
                    }) {
                        Ok(_) => (),
                        Err(e) => {
                            log::error!("Failed to send answer for {user_id}: {e}");
                            continue;
                        }
                    };

                    tokio::time::sleep(Duration::from_secs(10)).await;
                }

                let score = self
                    .db
                    .get_answers_statuses(&user_id)
                    .await
                    .unwrap_or_default()
                    .iter()
                    .filter(|x| **x == AnswerStatus::Correct)
                    .count() as u32;

                match self.db.set_score(&user_id, score).await {
                    Ok(_) => (),
                    Err(e) => {
                        log::error!("Failed to set score for {user_id}: {e}");
                        continue;
                    }
                };

                match tx.send(ServerMessage::GameEnd { score }) {
                    Ok(_) => (),
                    Err(e) => {
                        log::error!("Failed to send game end message to {user_id}: {e}");
                        continue;
                    }
                };
            }
        });

        tokio::select! {
            _ = receive_from_client => {}
            _ = wait_for_game_to_start => {}
            _ = send_to_client => {},
        };
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
