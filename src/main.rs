mod structs;

use crate::structs::*;
use axum::extract::State;
use axum::routing::post;
use axum::{http::StatusCode, routing::get, Json, Router};
use chrono::{TimeDelta, Utc};
use std::collections::{BinaryHeap, LinkedList};
use std::ops::Add;
use std::sync::{Arc, Mutex};
use tokio::main;

#[main]
async fn main() {
    let buy_order = Arc::new(Mutex::new(BinaryHeap::new()));
    let sell_order = Arc::new(Mutex::new(BinaryHeap::new()));
    let points_queue = Arc::new(Mutex::new(LinkedList::new()));
    let order_book = OrderBook::new(buy_order, sell_order, points_queue);
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/daily", get(get_daily))
        .route("/transaction", post(make_transaction))
        .route("/daily", post(get_daily))
        .with_state(order_book);

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

async fn make_transaction(
    State(order_book): State<OrderBook>,
    Json(transaction): Json<Transaction>,
) -> StatusCode {
    match transaction.operation() {
        Operation::Buy => {
            order_book.add_buy_order(transaction);
            order_book.execute();
            StatusCode::OK
        }
        Operation::Sell => {
            order_book.add_sell_order(transaction);
            order_book.execute();
            StatusCode::OK
        }
    }
}

async fn get_daily() -> (StatusCode, Json<TimeSeries>) {
    let b1 = Point::new(0.0, 10.0, 0.0, 10.0, 1);
    let b2 = Point::new(0.0, 10.0, 0.0, 10.0, 1);
    let b3 = Point::new(0.0, 10.0, 0.0, 10.0, 1);
    let b4 = Point::new(0.0, 10.0, 0.0, 10.0, 1);
    let start = Utc::now();
    let end = start.add(TimeDelta::minutes(1));
    (
        StatusCode::OK,
        Json(TimeSeries::new(
            TimeRange::Day,
            start,
            end,
            vec![b1, b2, b3, b4],
        )),
    )
}
