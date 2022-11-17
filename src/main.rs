use segon::{
    adapters::{Jwt, Notifier, RedisUsersDatabase, Schedular, ShaHasher, UuidGenerator},
    controllers::{GameController, UsersController},
    handlers::{
        login_handler, register_handler, websocket_handler, with_game_controller, with_json_body,
        with_users_controller,
    },
    request::{LoginRequest, RegisterRequest},
};
use warp::Filter;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    // init db
    let db = RedisUsersDatabase::new().await.unwrap();

    // init users controller
    let users_controller: UsersController<RedisUsersDatabase, ShaHasher, Jwt, UuidGenerator> =
        UsersController::new(db.clone());

    // init game controller
    let notifier = Notifier::new();
    let schedular = Schedular::new(notifier.clone()).await.unwrap();
    let game_controller = GameController::new(db, schedular.clone(), notifier);

    // POST /register
    let register_route = warp::path("register")
        .and(warp::post())
        .and(with_users_controller(users_controller.clone()))
        .and(with_json_body::<RegisterRequest>())
        .and_then(register_handler);

    // POST /login
    let login_route = warp::path("login")
        .and(warp::post())
        .and(with_users_controller(users_controller.clone()))
        .and(with_json_body::<LoginRequest>())
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
