use segon::{
    adapters::MemoryDatabase,
    controllers::authorize,
    models::PeerMap,
    ports::Database,
    warp_handlers::{login_handler, register_handler},
    ws::connect,
};
use std::sync::Arc;
use tokio::sync::mpsc::{self, UnboundedReceiver};
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};
use warp::{ws::Ws, Filter};

fn with_db(
    db: MemoryDatabase,
) -> impl Filter<Extract = (MemoryDatabase,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let db = MemoryDatabase::new();
    let schedular = JobScheduler::new().await.unwrap();
    let schedular_cloned = schedular.clone();
    let schedular_filter = warp::any().map(move || schedular_cloned.clone());

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
        .and(with_db(db.clone()))
        .and_then(login_handler);

    let peer_map = PeerMap::default();
    let peer_map_filter = warp::any().map(move || peer_map.clone());

    let (game_start_notifier, websocket_listener) = mpsc::unbounded_channel::<String>();
    let websocket_listener = Arc::new(Mutex::new(websocket_listener));
    let websocket_listener = warp::any().map(move || websocket_listener.clone());

    // GET /game -> websocket upgrade
    let chat = warp::path("game")
        .and(with_db(db))
        .and(warp::ws())
        .and(warp::path::param())
        .and(peer_map_filter)
        .and(schedular_filter)
        .and(websocket_listener)
        .and_then(websocket_handler);

    let routes = register_route.or(login_route).or(chat);

    let game_start_job = Job::new("1/60 * * * * *", move |_uuid, _l| {
        game_start_notifier.send("Game has started".into()).unwrap();
    })
    .unwrap();

    schedular.add(game_start_job).await.unwrap();

    schedular.start().await.unwrap();

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn websocket_handler(
    db: impl Database,
    ws: Ws,
    token: String,
    peer_map: PeerMap,
    schedular: JobScheduler,
    websocket_listener: Arc<Mutex<UnboundedReceiver<String>>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let authorized = authorize(token, db, segon::adapters::Jwt).await;
    match authorized {
        Ok(_) => {
            Ok(ws
                .on_upgrade(move |socket| connect(socket, peer_map, schedular, websocket_listener)))
        }
        Err(_) => {
            eprintln!("Unauthenticated user");
            Err(warp::reject::not_found())
        }
    }
}
