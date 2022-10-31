use crate::{
    adapters::{Jwt, MemoryDatabase, ShaHasher},
    controllers::{login, register},
    models::User,
};
use warp::{hyper::StatusCode, reply::Reply};

type WarpResult<T> = Result<T, std::convert::Infallible>;

pub async fn register_handler(user: User, db: MemoryDatabase) -> WarpResult<impl Reply> {
    match register(user, db, ShaHasher, Jwt).await {
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

pub async fn login_handler(user: User, db: MemoryDatabase) -> WarpResult<impl Reply> {
    match login(user, db, ShaHasher, Jwt).await {
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
