use futures_util::{SinkExt, StreamExt, TryFutureExt};
use segon::{
    adapters::{MemoryDatabase, ShaHasher, SimpleTokenGenerator},
    controllers::{login, register},
    models::User,
};
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};
use warp::{hyper::StatusCode, Filter, Reply};

type Users = Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<Message>>>>;
static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let db = MemoryDatabase::new();

    let json_body = warp::body::content_length_limit(1024 * 16).and(warp::body::json());

    // POST /register
    let register_route = warp::path("register")
        .and(warp::post())
        .and(json_body)
        .and(with_db(db.clone()))
        .and_then(register_handler);

    // POST /login
    let login_route = warp::path("login")
        .and(warp::post())
        .and(json_body)
        .and(with_db(db))
        .and_then(login_handler);

    let users = Users::default();
    // Turn our "state" into a new Filter...
    let users = warp::any().map(move || users.clone());

    // GET /chat -> websocket upgrade
    let chat = warp::path("game")
        // The `ws()` filter will prepare Websocket handshake...
        .and(warp::ws())
        .and(users)
        .map(|ws: warp::ws::Ws, users| {
            // This will call our function if the handshake succeeds.
            ws.on_upgrade(move |socket| user_connected(socket, users))
        });

    let routes = register_route.or(login_route).or(chat);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

fn with_db(
    db: MemoryDatabase,
) -> impl Filter<Extract = (MemoryDatabase,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

async fn register_handler(
    user: User,
    db: MemoryDatabase,
) -> Result<impl warp::reply::Reply, std::convert::Infallible> {
    match register(user, db, ShaHasher, SimpleTokenGenerator).await {
        Ok(token) => {
            let response = warp::reply::json(&serde_json::json!({
                "status": "OK",
                "token": token
            }));
            Ok(warp::reply::with_status(response, StatusCode::CREATED))
        }
        Err(err) => {
            let response = warp::reply::json(&serde_json::json!({
                "status": "ERROR",
                "message": err.to_string(),
            }));

            Ok(warp::reply::with_status(
                response,
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

async fn login_handler(
    user: User,
    db: MemoryDatabase,
) -> Result<impl Reply, std::convert::Infallible> {
    match login(user, db, ShaHasher, SimpleTokenGenerator).await {
        Ok(token) => {
            let response = warp::reply::json(&serde_json::json!({
                "status": "OK",
                "token": token
            }));

            Ok(warp::reply::with_status(response, StatusCode::OK))
        }
        Err(err) => {
            let response = warp::reply::json(&serde_json::json!({
                "status": "UNAUTHORIZED",
                "error": err.to_string(),
            }));
            Ok(warp::reply::with_status(response, StatusCode::UNAUTHORIZED))
        }
    }
}

async fn user_connected(ws: WebSocket, users: Users) {
    // Use a counter to assign a new unique ID for this user.
    let my_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);

    eprintln!("new chat user: {}", my_id);

    // Split the socket into a sender and receive of messages.
    let (mut user_ws_tx, mut user_ws_rx) = ws.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the websocket...
    let (tx, rx) = mpsc::unbounded_channel();
    let mut rx = UnboundedReceiverStream::new(rx);

    tokio::task::spawn(async move {
        while let Some(message) = rx.next().await {
            user_ws_tx
                .send(message)
                .unwrap_or_else(|e| {
                    eprintln!("websocket send error: {}", e);
                })
                .await;
        }
    });

    // Save the sender in our list of connected users.
    users.write().await.insert(my_id, tx);

    // Return a `Future` that is basically a state machine managing
    // this specific user's connection.

    // Every time the user sends a message, broadcast it to
    // all other users...
    while let Some(result) = user_ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("websocket error(uid={}): {}", my_id, e);
                break;
            }
        };
        user_message(my_id, msg, &users).await;
    }

    // user_ws_rx stream will keep processing as long as the user stays
    // connected. Once they disconnect, then...
    user_disconnected(my_id, &users).await;
}

async fn user_message(my_id: usize, msg: Message, users: &Users) {
    // Skip any non-Text messages...
    let msg = if let Ok(s) = msg.to_str() {
        s
    } else {
        return;
    };

    let new_msg = format!("<User#{}>: {}", my_id, msg);

    // New message from this user, send it to everyone else (except same uid)...
    for (&uid, tx) in users.read().await.iter() {
        if my_id != uid {
            if let Err(_disconnected) = tx.send(Message::text(new_msg.clone())) {
                // The tx is disconnected, our `user_disconnected` code
                // should be happening in another task, nothing more to
                // do here.
            }
        }
    }
}

async fn user_disconnected(my_id: usize, users: &Users) {
    eprintln!("good bye user: {}", my_id);

    // Stream closed up, so remove from the user list
    users.write().await.remove(&my_id);
}

