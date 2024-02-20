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
        .route("/daily", post(get_daily));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
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
