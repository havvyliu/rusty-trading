use axum::extract::{Query, State};
use axum::routing::post;
use axum::{http::StatusCode, routing::get, Json, Router};
use chrono::{DateTime, TimeDelta, Utc};
use external::IntradayStock;
use rand::Rng;
use reqwest::Client;
use tower_http::cors::CorsLayer;
use std::collections::{BinaryHeap, HashMap, LinkedList};
use std::env;
use std::ops::Add;
use std::sync::{Arc, Mutex, RwLock};
use tokio::main;
use tokio_cron_scheduler::{Job, JobScheduler};
use rusty_trading_model::structs::*;
use dashmap::DashMap;

mod external;
mod simulation;
mod execution;

#[main]
async fn main() {
    let order_book_map: Arc<DashMap<String, OrderBook>> = Arc::new(DashMap::new());

    schedule_cron_job(order_book_map.clone()).await;

    let client = reqwest::Client::new();
    let time_series: Arc<Mutex<TimeSeries>> = Arc::new(Mutex::new(TimeSeries::default()));

    let app = Router::new()
        .route("/stock", get(get_stock))
        .route("/transaction", post(make_transaction))
        .route("/simulate_v2", post(simulate_v2))
        .layer(CorsLayer::permissive())
        .with_state(order_book_map)
        .route("/third_party", get(get_real_data))
        .layer(CorsLayer::permissive())
        .with_state(client);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}


async fn schedule_cron_job(order_book_map: Arc<DashMap<String, OrderBook>>) {
    // add scheduler
    let scheduler = JobScheduler::new().await.unwrap();

    let job = Job::new_async("1/60 * * * * *", move |_uuid, _l| {
        let map_arc_clone = order_book_map.clone();
        Box::pin(
            execution::flush(map_arc_clone)
        )
    });
    let _ = scheduler.add(job.unwrap()).await;
    // spawn another thread to process background tasks
    tokio::spawn(async move { scheduler.start().await });
}

async fn make_transaction(
    State(order_book_map): State<Arc<DashMap<String, OrderBook>>>,
    Json(transaction): Json<Transaction>,
) -> StatusCode {
    let symbol = transaction.symbol();
    let order_book = match order_book_map.get(symbol) {
        Some(value) => value,
        None => {
            let buy_order = Arc::new(Mutex::new(BinaryHeap::new()));
            let sell_order = Arc::new(Mutex::new(BinaryHeap::new()));
            let points_queue = Arc::new(RwLock::new(LinkedList::new()));
            let start_point = Arc::new(Mutex::new(Point::blank()));
            let new_order_book = OrderBook::new(buy_order, sell_order, points_queue, start_point);
            order_book_map.insert(symbol.to_string(), new_order_book.to_owned());
            order_book_map.get(symbol).unwrap()
        }
    };
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

async fn simulate_v2(
    State(order_book_arc): State<Arc<DashMap<String, OrderBook>>>,
    Query(params): Query<HashMap<String, String>>,
) -> StatusCode {
    let symbol = params.get("stock").unwrap();
    let time_series = match order_book_arc.get_mut(symbol.as_str()) {
        Some(entry) => entry.value().time_series(),
        None => {
            let order_b = OrderBook::default();
            let ts_clone = order_b.time_series().clone();
            order_book_arc.insert(symbol.clone(), order_b);
            ts_clone
        }
    };
    let mut start_price = 200.0;
    time_series.write().unwrap().update_time_range_unit(TimeRange::Minute);
    if (time_series.write().unwrap().data().len() >= 1000) {
        log::info!("Stop generating new points");
        return StatusCode::OK;
    }
    let mut timestamp = Utc::now();
    for _ in 0..100 {
        let next_price = simulation::algo::down_and_up(start_price);
        let size = time_series.write().unwrap().data().len();
        timestamp = timestamp.checked_add_signed(TimeDelta::days(1)).unwrap();
        time_series.write().unwrap().data().insert(size, 
            Point::new_with_timestamp(start_price, next_price * 1.1, next_price * 0.9, next_price, 100, 
                timestamp.clone()));
        start_price = next_price;
    }
    println!("Time series size is {}", time_series.write().unwrap().data().len());
    
    StatusCode::OK
}


async fn get_real_data(State(client): State<Client>) -> (StatusCode, Json<TimeSeries>) {
    let api_key = match env::var("API_KEY") {
        Ok(api_key) => api_key,
        Err(e) => {
            println!("No API_KEY set in env...");
            return (StatusCode::BAD_REQUEST, Json(TimeSeries::default()));
        },
    };
    let url = format!("https://www.alphavantage.co/query?function=TIME_SERIES_INTRADAY&symbol=NVDA&interval=5min&apikey={}", api_key);
    let reqwest_response = match client.get(url).send().await {
            Ok(res) => res,
            Err(_err) => {
                return (StatusCode::BAD_REQUEST, Json(TimeSeries::default()));
            }
        };
    let body = reqwest_response.text().await.unwrap();
    let intraday: IntradayStock = serde_json::from_str(&body).unwrap();
    let map = intraday.get_points_map().to_owned();
    let start = map.keys().min().unwrap().to_owned();
    let end = map.keys().max().unwrap().to_owned();
    let all_points: Vec<Point> = map.into_iter()
        .map(|(_, val)| val)
        .collect();
    (StatusCode::OK, Json(
        TimeSeries::new(TimeRange::Minute, start, end, all_points)
    ))
}

async fn get_stock(
    Query(params): Query<HashMap<String, String>>,
    State(order_book_map): State<Arc<DashMap<String, OrderBook>>>)
    -> (StatusCode, Json<TimeSeries>) {
    let stock_name = params.get("stock").unwrap();
    println!("GET stock called.. for {}", stock_name);
    
    
    let time_series = match order_book_map.get(stock_name) {
        Some(order_book_ref) => {
            // order_book_ref.update_time_series();
            
            let time_series_arc = order_book_ref.time_series();
            let time_series_guard = time_series_arc.read().unwrap();
            (*time_series_guard).clone()
        },
        None => {
            // Return empty TimeSeries if stock not found
            TimeSeries::default()
        }
    };

    (
        StatusCode::OK,
        Json(time_series),
    )
}


#[tokio::test]
async fn test_flush_working() {
    let buy_order = Arc::new(Mutex::new(BinaryHeap::new()));
    let sell_order = Arc::new(Mutex::new(BinaryHeap::new()));
    let points_queue = Arc::new(RwLock::new(LinkedList::new()));
    let start_point = Arc::new(Mutex::new(Point::blank()));
    let new_order_book = OrderBook::new(buy_order, sell_order, points_queue, start_point);
    let order_book_map = DashMap::new();
    order_book_map.insert("NVDA".to_string(), new_order_book);
    let order_book_arc = Arc::new(order_book_map);

    for i in 0..5 {
        {
            let clone = order_book_arc.clone();
            let order_book = clone.get("NVDA").unwrap();
            let price = (i + 1) as f64 * 100.0;
            let buy = Transaction::buy("NVDA".to_string(), price, 1000);
            let sell = Transaction::sell("NVDA".to_string(), price, 1000);
            order_book.add_buy_order(buy);
            order_book.add_sell_order(sell);
            order_book.execute();
            assert_eq!(order_book.points().read().unwrap().len(), i);
            assert_eq!(order_book.cur_point().lock().unwrap().close, price);
        }
        //drop(map);
        execution::flush(order_book_arc.clone()).await;
        println!("Here");
        {
            let clone = order_book_arc.clone();
            let order_book = clone.get("NVDA").unwrap();
            assert_eq!(order_book.points().read().unwrap().len(), i + 1);
        }
    }
}
