use crate::{
    adapters::{Jwt, MemoryDatabase, ShaHasher},
    controllers::UsersController,
    models::{GameStartSignalReceiver, PeerMap, User},
    ws::connect,
};
use tokio_cron_scheduler::JobScheduler;
use warp::{hyper::StatusCode, reply::Reply, ws::Ws, Filter};

type WarpResult<T> = Result<T, std::convert::Infallible>;
type Controller = UsersController<MemoryDatabase, ShaHasher, Jwt>;

pub fn with_users_controller(
    controller: Controller,
) -> impl Filter<Extract = (Controller,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || controller.clone())
}

pub async fn register_handler(controller: Controller, user: User) -> WarpResult<impl Reply> {
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

pub async fn login_handler(controller: Controller, user: User) -> WarpResult<impl Reply> {
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

pub async fn websocket_handler(
    controller: Controller,
    ws: Ws,
    token: String,
    peer_map: PeerMap,
    schedular: JobScheduler,
    game_start_signal_reciever: GameStartSignalReceiver,
) -> Result<impl warp::Reply, warp::Rejection> {
    let authorized = controller.authorize(token).await;
    match authorized {
        Ok(_) => Ok(ws.on_upgrade(move |socket| {
            connect(socket, peer_map, schedular, game_start_signal_reciever)
        })),
        Err(_) => {
            eprintln!("Unauthenticated user");
            Err(warp::reject::not_found())
        }
    }
}
