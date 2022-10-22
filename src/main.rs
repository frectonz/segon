use segon::{
    adapters::{MemoryDatabase, ShaHasher},
    controllers::register,
    models::User,
};
use std::convert::Infallible;
use warp::{http::StatusCode, Filter};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let db = MemoryDatabase::new();
    let users_route = warp::path!("users")
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 16).and(warp::body::json()))
        .and(with_db(db))
        .and_then(register_handler);

    warp::serve(users_route).run(([127, 0, 0, 1], 3030)).await;
}

fn with_db(
    db: MemoryDatabase,
) -> impl Filter<Extract = (MemoryDatabase,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

pub async fn register_handler(
    user: User,
    db: MemoryDatabase,
) -> Result<impl warp::Reply, Infallible> {
    let hasher = ShaHasher::default();
    register(user, db, hasher).await.unwrap();

    Ok(StatusCode::CREATED)
}
