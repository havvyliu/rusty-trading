mod structs;

use std::ops::Add;
use axum::{
    http::StatusCode,
    Json,
    routing::get, Router
};
use axum::routing::post;
use chrono::{TimeDelta, Utc};
use crate::structs::*;

#[tokio::main]
async fn main() {

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/daily", get(get_daily))
        .route("/transaction", post(make_transaction))
        .route("/daily", post(get_daily));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

//todo: Onboard DB
//todo: Implement fn to flush transactions to time series data?
//todo: Implement fn to flush changes caused by transactions to db?
//todo: Implement a trading engine? Execute orders from buy and sell side
//todo(ui): {
// 1. UI, display a stock graph
// 2. Dynamically update the graph (without user refresh)
// 3.
// }

async fn make_transaction(Json(transaction): Json<Transaction>) -> StatusCode {
    match transaction.operation() {
        Operation::Buy => {
            StatusCode::OK
        }
        Operation::Sell => {StatusCode::OK}
        _ => StatusCode::BAD_GATEWAY
    }
}

async fn set_point(Json(payload): Json<Point>) -> StatusCode {
    StatusCode::OK
}

async fn get_daily() -> (StatusCode, Json<TimeSeries>) {
    let b1 = Point::new(0, 10, 0, 10, 1);
    let b2 = Point::new(0, 10, 0, 10, 1);
    let b3 = Point::new(0, 10, 0, 10, 1);
    let b4 = Point::new(0, 10, 0, 10, 1);
    let start = Utc::now();
    let end = start.add(TimeDelta::minutes(1));
    (StatusCode::OK, 
        Json(TimeSeries::new(TimeRange::Day, start, end, vec![b1, b2, b3, b4])))
}
