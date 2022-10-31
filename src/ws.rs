use crate::models::{PeerMap, NEXT_USER_ID};
use futures_util::{future, StreamExt, TryStreamExt};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};
use tokio_cron_scheduler::JobScheduler;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};

pub async fn connect(
    ws: WebSocket,
    peer_map: PeerMap,
    mut schedular: JobScheduler,
    websocket_listener: Arc<tokio::sync::Mutex<UnboundedReceiver<String>>>,
) {
    // Bookkeeping
    let my_id = NEXT_USER_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    println!("Welcome User {}", my_id);

    // Establishing a connection
    let (outgoing, incoming) = ws.split();

    // Insert the write part of this peer to the peer map.
    let (tx, rx) = unbounded_channel();
    let rx = UnboundedReceiverStream::new(rx);

    let message = if let Ok(Some(time)) = schedular.time_till_next_job().await {
        format!("{}s till game starts", time.as_secs())
    } else {
        "Welcome".into()
    };

    tx.send(Message::text(message)).unwrap();
    peer_map.lock().unwrap().insert(my_id, tx);

    // let p_map = peer_map.clone();
    let receive_from_client = incoming.try_for_each(|msg| {
        println!(
            "Received a message from {}: {}",
            my_id,
            msg.to_str().unwrap()
        );

        future::ok(())
    });

    let send_to_client = rx.map(Ok).forward(outgoing);

    let p_map = peer_map.clone();
    let wait_for_game_to_start = Box::pin(async move {
        let mut websocket_listener = websocket_listener.lock().await;

        while let Some(_) = websocket_listener.recv().await {
            let questions = vec!["hello 1", "hello 2", "hello 3"];

            for (i, question) in questions.into_iter().enumerate() {
                let p_map = p_map.clone();

                tokio::spawn(async move {
                    let sleep_time: u64 = i as u64 * 10;

                    tokio::time::sleep(Duration::from_secs(sleep_time)).await;
                    let p_map = p_map.lock().unwrap();

                    for (_, tx) in p_map.iter() {
                        tx.send(Message::text(question)).unwrap();
                    }
                });
            }

            // println!("received game start message");
            // let p_map = p_map.lock().unwrap();
        }
    });

    // pin_mut!(receive_from_client, send_to_client);
    tokio::select! {
        _ = receive_from_client => {}
        _ = wait_for_game_to_start => {}
        _ = send_to_client => {},
    }

    println!("{} disconnected", &my_id);
    peer_map.lock().unwrap().remove(&my_id);
}

// add them to a list of connected user -> done
// tell the client how much time is left until the game starts -> done
// when the timer reaches the game time send a start signal and the first question to the client
// receive answers for 10 seconds until the question timer ends
// send the answer to all of the connected clients and whether or not they were correct
// drop off the clients that have answered wrong from the answerers list
// repeat this process until all questions have been answered
// assemble a leaderboard and send it to every client with each client's rank on the leaderboard
