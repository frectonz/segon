use crate::{
    controllers::{GameController, UsersController},
    models::User,
    ports::{GameDatabase, GameStartNotifier, Hasher, JobSchedular, TokenGenerator, UsersDatabase},
};
use warp::{hyper::StatusCode, reply::Reply, ws::Ws, Filter};

type WarpResult<T> = Result<T, std::convert::Infallible>;

pub fn with_users_controller<
    D: UsersDatabase + Clone + Send + Sync,
    H: Hasher + Clone + Send + Sync,
    T: TokenGenerator + Clone + Send + Sync,
>(
    controller: UsersController<D, H, T>,
) -> impl Filter<Extract = (UsersController<D, H, T>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || controller.clone())
}

pub fn with_game_controller<
    GD: GameDatabase + Send + Sync + Clone + 'static,
    JS: JobSchedular + Send + Sync + Clone + 'static,
    GSN: GameStartNotifier + Send + Sync + Clone + 'static,
>(
    controller: GameController<GD, JS, GSN>,
) -> impl Filter<Extract = (GameController<GD, JS, GSN>,), Error = std::convert::Infallible> + Clone
{
    warp::any().map(move || controller.clone())
}

pub async fn register_handler<
    D: UsersDatabase + Clone,
    H: Hasher + Clone + Send,
    T: TokenGenerator + Clone,
>(
    controller: UsersController<D, H, T>,
    user: User,
) -> WarpResult<impl Reply> {
    match controller.register(user).await {
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

pub async fn login_handler<
    D: UsersDatabase + Clone,
    H: Hasher + Clone,
    T: TokenGenerator + Clone,
>(
    controller: UsersController<D, H, T>,
    user: User,
) -> WarpResult<impl Reply> {
    match controller.login(user).await {
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

pub async fn websocket_handler<
    D: UsersDatabase + Clone,
    H: Hasher + Clone,
    T: TokenGenerator + Clone,
    GD: GameDatabase + Send + Sync + Clone + 'static,
    JS: JobSchedular + Send + Sync + Clone + 'static,
    GSN: GameStartNotifier + Send + Sync + Clone + 'static,
>(
    controller: UsersController<D, H, T>,
    game_controller: GameController<GD, JS, GSN>,
    ws: Ws,
    token: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    let authorized = controller.authorize(token).await;
    match authorized {
        Ok(user) => Ok(ws.on_upgrade(move |socket| game_controller.start(user, socket))),
        Err(_) => {
            eprintln!("Unauthenticated user");
            Err(warp::reject::not_found())
        }
    }
}
