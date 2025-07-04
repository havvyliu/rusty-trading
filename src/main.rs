use axum::extract::{Query, State};
use axum::routing::post;
use axum::{http::StatusCode, routing::get, Json, Router};
use chrono::{TimeDelta, Utc};
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

#[main]
async fn main() {
    let order_book_map: Arc<DashMap<String, OrderBook>> = Arc::new(DashMap::new());

    schedule_cron_job(order_book_map.clone()).await;

    let client = reqwest::Client::new();

    let app = Router::new()
        .route("/daily", get(get_daily))
        .route("/transaction", post(make_transaction))
        .route("/simulate", post(simulate))
        .layer(CorsLayer::permissive())
        .with_state(order_book_map)
        .route("/third_party", get(get_real_data))
        .layer(CorsLayer::permissive())
        .with_state(client);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn flush(order_book_map: Arc<DashMap<String, OrderBook>>) {
    let map = order_book_map.clone();
    for ref_multi in map.iter() {
        let order_book = ref_multi.value();
        let points_lock = order_book.points();
        let mut points_queue = points_lock.write().unwrap();
        let cur_point = order_book.cur_point();
        let cur_pt: Point = cur_point.lock().unwrap().to_owned();
        let mut cur_pt_lock = cur_point.lock().unwrap();
        let close_val = cur_pt.clone().close;
        *cur_pt_lock = Point::new(close_val, close_val, close_val, close_val, 0);
        println!("writing point {:?} to queue", cur_pt);
        points_queue.push_back(cur_pt);
        println!("queue size is {:?}", points_queue.len());
    }

}

async fn schedule_cron_job(order_book_map: Arc<DashMap<String, OrderBook>>) {
    // add scheduler
    let scheduler = JobScheduler::new().await.unwrap();

    let job = Job::new_async("1/60 * * * * *", move |_uuid, _l| {
        let map_arc_clone = order_book_map.clone();
        Box::pin(
            flush(map_arc_clone)
        )
    });
    let _ = scheduler.add(job.unwrap()).await;
    // spawn another thread to process background tasks
    tokio::spawn(async move { scheduler.start().await });
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

async fn simulate(
    State(order_book_arc): State<Arc<DashMap<String, OrderBook>>>,
    symbol: String,
) -> StatusCode {
    println!("Start simulating..");
    let order_book = match order_book_arc.get(&symbol) {
        Some(value) => value,
        None => {
            let buy_order = Arc::new(Mutex::new(BinaryHeap::new()));
            let sell_order = Arc::new(Mutex::new(BinaryHeap::new()));
            let points_queue = Arc::new(RwLock::new(LinkedList::new()));
            let start_point = Arc::new(Mutex::new(Point::blank()));
            let new_order_book = OrderBook::new(buy_order, sell_order, points_queue, start_point);
            order_book_arc.insert(symbol.clone(), new_order_book.to_owned());
            order_book_arc.get(&symbol).unwrap()
        }
    };
    for _i in 1..10 {
        let price = rand::thread_rng().gen_range(50..=100) as f32;
        let amount = rand::thread_rng().gen_range(100..=500);
        //add buy orders.
        let buy_order = Transaction::buy(symbol.clone(), price, amount);
        order_book.add_buy_order(buy_order);
        //add sell orders.
        let sell_order = Transaction::sell(symbol.clone(), price, amount);
        order_book.add_sell_order(sell_order);
    };
    order_book.execute();
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
    // TODO: parse the response to our model
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

async fn get_daily(Query(params): Query<HashMap<String, String>>, State(order_book_map): State<Arc<DashMap<String, OrderBook>>>) -> (StatusCode, Json<TimeSeries>) {
    println!("get_daily called..");
    let start = Utc::now();
    let end = start.add(TimeDelta::minutes(1));
    let stock_name = params.get("stock").unwrap();
    println!("size is {:?}", order_book_map.len());
    let points_copy = match order_book_map.get(stock_name) {
        Some(value) => {
            let mut points_copy = vec![];
            let write_lock = value.points();
            let points = write_lock.read().unwrap();
            for point in &*points {
                points_copy.push(point.clone());
            }
            points_copy
        },
        _ => {
            vec![]
        }
    };

    (
        StatusCode::OK,
        Json(TimeSeries::new(
            TimeRange::Day,
            start,
            end,
            points_copy,
        )),
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
            let price = (i + 1) as f32 * 100.0;
            let buy = Transaction::buy("NVDA".to_string(), price, 1000);
            let sell = Transaction::sell("NVDA".to_string(), price, 1000);
            order_book.add_buy_order(buy);
            order_book.add_sell_order(sell);
            order_book.execute();
            assert_eq!(order_book.points().read().unwrap().len(), i);
            assert_eq!(order_book.cur_point().lock().unwrap().close, price);
        }
        //drop(map);
        flush(order_book_arc.clone()).await;
        println!("Here");
        {
            let clone = order_book_arc.clone();
            let order_book = clone.get("NVDA").unwrap();
            assert_eq!(order_book.points().read().unwrap().len(), i + 1);
        }
    }
}
