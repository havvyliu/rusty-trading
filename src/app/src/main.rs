use axum::extract::State;
use axum::routing::post;
use axum::{http::StatusCode, routing::get, Json, Router};
use chrono::{TimeDelta, Utc};
use std::collections::{BinaryHeap, LinkedList};
use std::ops::Add;
use std::sync::{Arc, Mutex, RwLock};
use tokio::main;
use tokio_cron_scheduler::{Job, JobScheduler};
use rusty_trading_lib::structs::*;


#[main]
async fn main() {
    let buy_order = Arc::new(Mutex::new(BinaryHeap::new()));
    let sell_order = Arc::new(Mutex::new(BinaryHeap::new()));
    let points_queue = Arc::new(RwLock::new(LinkedList::new()));
    let start_point = Arc::new(Mutex::new(Point::blank()));
    let order_book = OrderBook::new(buy_order, sell_order, points_queue.clone(), start_point);
    let cur_point = order_book.cur_point();

    // add scheduler
    let scheduler = JobScheduler::new().await.unwrap();

    let job = Job::new_async("1/60 * * * * *", move |_uuid, _l| {
        let points_queue = points_queue.clone();
        let cur_point = cur_point.clone();
        Box::pin(async move {
            let cur_pt: Point = cur_point.lock().unwrap().to_owned();
            println!("writing point {:?} to queue", cur_pt);
            points_queue.write().unwrap().push_back(cur_pt);
        })
    });
    let _ = scheduler.add(job.unwrap()).await;
    // spawn another thread to process background tasks
    tokio::spawn(async move { scheduler.start().await });

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/daily", get(get_daily))
        .route("/transaction", post(make_transaction))
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

async fn get_daily(State(order_book): State<OrderBook>) -> (StatusCode, Json<TimeSeries>) {
    let start = Utc::now();
    let end = start.add(TimeDelta::minutes(1));
    let points = order_book.points();
    (
        StatusCode::OK,
        Json(TimeSeries::new(
            TimeRange::Day,
            start,
            end,
            points.into_iter().collect(),
        )),
    )
}
