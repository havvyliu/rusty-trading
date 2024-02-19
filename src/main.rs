use std::time::{Duration, SystemTime};
use axum::{
    http::StatusCode,
    Json,
    routing::get, Router
};
use rusty_trading::*;

#[tokio::main]
async fn main() {

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/daily", get(get_daily));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}


async fn get_daily() -> (StatusCode, Json<TimeSeries>) {
    let b1 = Block::new(0, 10, 0, 10, 1);
    let b2 = Block::new(0, 10, 0, 10, 1);
    let b3 = Block::new(0, 10, 0, 10, 1);
    let b4 = Block::new(0, 10, 0, 10, 1);
    let start = SystemTime::now();
    let end = start.checked_add(Duration::new(60, 0));
    (StatusCode::OK, 
        Json(TimeSeries::new(TimeRange::Daily, start, end.unwrap(), vec![b1, b2, b3, b4])))
}
