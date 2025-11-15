use std::sync::Arc;
use dashmap::DashMap;
use rusty_trading_model::structs::{OrderBook, Point};

pub async fn flush(order_book_map: Arc<DashMap<String, OrderBook>>) {
    
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