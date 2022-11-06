use segon::{
    adapters::{Jwt, MemoryDatabase, Notifier, PeerMap, Schedular, ShaHasher},
    controllers::{GameController, UsersController},
    warp_handlers::{
        login_handler, register_handler, websocket_handler, with_game_controller,
        with_users_controller,
    },
};
use warp::Filter;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let users_controller = UsersController::new(MemoryDatabase::new(), ShaHasher, Jwt);

    let notifier = Notifier::new();
    let schedular = Schedular::new(&notifier).await;
    let game_controller = GameController::new(
        MemoryDatabase::new(),
        PeerMap::default(),
        schedular,
        notifier
    );

    let json_body = warp::body::content_length_limit(1024 * 16).and(warp::body::json());

    // POST /register
    let register_route = warp::path("register")
        .and(warp::post())
        .and(with_users_controller(users_controller.clone()))
        .and(json_body)
        .and_then(register_handler);

    // POST /login
    let login_route = warp::path("login")
        .and(warp::post())
        .and(with_users_controller(users_controller.clone()))
        .and(json_body)
        .and_then(login_handler);

    // let peer_map = crate::models::PeerMap::default();
    // let peer_map_filter = warp::any().map(move || peer_map.clone());

    // let (game_start_notifier, game_start_signal_reciever) = mpsc::unbounded_channel::<()>();
    // let game_start_signal_reciever = Arc::new(Mutex::new(game_start_signal_reciever));
    // let game_start_signal_reciever = warp::any().map(move || game_start_signal_reciever.clone());

    // GET /game -> websocket upgrade
    let chat = warp::path("game")
        .and(with_users_controller(users_controller))
        .and(with_game_controller(game_controller))
        .and(warp::ws())
        .and(warp::path::param())
        .and_then(websocket_handler);

    let routes = register_route.or(login_route).or(chat);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
