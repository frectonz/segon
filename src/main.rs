use segon::{
    adapters::{
        GameMemoryDatabase, Jwt, Notifier, Schedular, ShaHasher, UsersMemoryDatabase, UuidGenerator,
    },
    controllers::{GameController, UsersController},
    handlers::{
        login_handler, register_handler, websocket_handler, with_game_controller, with_json_body,
        with_users_controller,
    },
    request::{LoginRequest, RegisterRequest},
};
use std::convert::Infallible;
use warp::{hyper::StatusCode, Filter, Rejection, Reply};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST", "DELETE"])
        .allow_headers(vec!["Content-Length", "Content-Type"]);

    // init db
    // let db = RedisUsersDatabase::new().await.unwrap();

    // init users controller
    let users_controller: UsersController<UsersMemoryDatabase, ShaHasher, Jwt, UuidGenerator> =
        UsersController::new(UsersMemoryDatabase::new());

    // init game controller
    let notifier = Notifier::new();
    let schedular = Schedular::new(notifier.clone()).await.unwrap();
    let game_controller =
        GameController::new(GameMemoryDatabase::default(), schedular.clone(), notifier);

    // POST /register
    let register_route = warp::path("register")
        .and(warp::post())
        .and(with_users_controller(users_controller.clone()))
        .and(with_json_body::<RegisterRequest>())
        .and_then(register_handler)
        .map(|ok| ok);

    // POST /login
    let login_route = warp::path("login")
        .and(warp::post())
        .and(with_users_controller(users_controller.clone()))
        .and(with_json_body::<LoginRequest>())
        .and_then(login_handler)
        .map(|ok| ok);

    // GET /game -> websocket upgrade
    let chat = warp::path("game")
        .and(with_users_controller(users_controller))
        .and(with_game_controller(game_controller))
        .and(warp::ws())
        .and(warp::path::param())
        .and_then(websocket_handler)
        .map(|ok| ok);

    // Serve build directory
    // let serve = warp::fs::dir("client/build");

    let routes = register_route
        .or(login_route)
        .or(chat)
        // .or(serve)
        .recover(handle_rejection)
        .with(cors);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn handle_rejection(_: Rejection) -> Result<impl Reply, Infallible> {
    let response = warp::reply::json(&serde_json::json!({
        "message": "error"
    }));
    Ok(warp::reply::with_status(
        response,
        StatusCode::INTERNAL_SERVER_ERROR,
    ))
}
