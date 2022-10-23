use segon::{
    adapters::{MemoryDatabase, ShaHasher},
    controllers::{login, register},
    models::User,
};
use warp::{hyper::StatusCode, Filter, Reply};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let db = MemoryDatabase::new();

    let json_body = warp::body::content_length_limit(1024 * 16).and(warp::body::json());

    // POST /register
    let register_route = warp::path!("register")
        .and(warp::post())
        .and(json_body)
        .and(with_db(db.clone()))
        .and_then(register_handler);

    // POST /login
    let login_route = warp::path!("login")
        .and(warp::post())
        .and(json_body)
        .and(with_db(db))
        .and_then(login_handler);

    let routes = register_route.or(login_route);

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
    let hasher = ShaHasher::default();

    match register(user, db, hasher).await {
        Ok(_) => {
            let response = warp::reply::json(&serde_json::json!({
                "status": "OK",
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
    let hasher = ShaHasher::default();
    match login(user, db, hasher).await {
        Ok(user) => {
            let response = warp::reply::json(&serde_json::json!({
                "status": "OK",
                "data": user
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
