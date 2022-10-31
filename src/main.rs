use segon::{
    adapters::{Jwt, MemoryDatabase, ShaHasher},
    controllers::UsersController,
    models::PeerMap,
    warp_handlers::{login_handler, register_handler, websocket_handler, with_users_controller},
};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};
use warp::Filter;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let controller = UsersController::new(MemoryDatabase::new(), ShaHasher, Jwt);

    let schedular = JobScheduler::new().await.unwrap();
    let schedular_cloned = schedular.clone();
    let schedular_filter = warp::any().map(move || schedular_cloned.clone());

    let json_body = warp::body::content_length_limit(1024 * 16).and(warp::body::json());

    // POST /register
    let register_route = warp::path("register")
        .and(warp::post())
        .and(with_users_controller(controller.clone()))
        .and(json_body)
        .and_then(register_handler);

    // POST /login
    let login_route = warp::path("login")
        .and(warp::post())
        .and(with_users_controller(controller.clone()))
        .and(json_body)
        .and_then(login_handler);

    let peer_map = PeerMap::default();
    let peer_map_filter = warp::any().map(move || peer_map.clone());

    let (game_start_notifier, game_start_signal_reciever) = mpsc::unbounded_channel::<String>();
    let game_start_signal_reciever = Arc::new(Mutex::new(game_start_signal_reciever));
    let game_start_signal_reciever = warp::any().map(move || game_start_signal_reciever.clone());

    // GET /game -> websocket upgrade
    let chat = warp::path("game")
        .and(with_users_controller(controller))
        .and(warp::ws())
        .and(warp::path::param())
        .and(peer_map_filter)
        .and(schedular_filter)
        .and(game_start_signal_reciever)
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
