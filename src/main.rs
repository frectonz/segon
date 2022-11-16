use segon::{
    adapters::{Jwt, Notifier, RedisUsersDatabase, Schedular, ShaHasher},
    controllers::{GameController, UsersController},
    handlers::{
        login_handler, register_handler, websocket_handler, with_game_controller,
        with_users_controller,
    },
};
use warp::Filter;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let db = RedisUsersDatabase::new().await;
    let users_controller = UsersController::new(db.clone(), ShaHasher, Jwt);

    let notifier = Notifier::new();
    let schedular = Schedular::new(&notifier).await;
    let game_controller = GameController::new(db, schedular.clone(), notifier);

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
