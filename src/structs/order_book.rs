use std::{
    borrow::{Borrow, BorrowMut},
    cmp::min,
    collections::{BinaryHeap, LinkedList},
    sync::{Arc, Mutex, RwLock},
};

use super::{Point, Transaction};

// use crate::{Point, Transaction};

#[derive(Clone)]
pub struct OrderBook {
    buy_order: Arc<Mutex<BinaryHeap<Transaction>>>,
    sell_order: Arc<Mutex<BinaryHeap<Transaction>>>,
    points_queue: Arc<RwLock<LinkedList<Point>>>,
    cur_point: Arc<Mutex<Point>>,
}

impl OrderBook {
    pub fn new(
        buy_order: Arc<Mutex<BinaryHeap<Transaction>>>,
        sell_order: Arc<Mutex<BinaryHeap<Transaction>>>,
        points_queue: Arc<RwLock<LinkedList<Point>>>,
        cur_point: Arc<Mutex<Point>>,
    ) -> Self {
        Self {
            buy_order,
            sell_order,
            points_queue,
            cur_point,
        }
    }

    pub fn add_buy_order(&self, buy_order: Transaction) {
        self.buy_order.lock().unwrap().push(buy_order);
    }

    pub fn add_sell_order(&self, sell_order: Transaction) {
        self.sell_order.lock().unwrap().push(sell_order);
    }

    pub fn points(&self) -> Arc<RwLock<LinkedList<Point>>> {
        self.points_queue.clone()
    }

    pub fn points_mut(&self) -> LinkedList<Point> {
        let write_lock = self.points_queue.write().unwrap();
        write_lock.clone()
    }

    pub fn cur_point(&self) -> Arc<Mutex<Point>> {
        Arc::clone(&self.cur_point)
    }

    pub fn execute(&self) {
        println!("Start executing...");
        let mut p = self.cur_point.lock().unwrap();
        let mut buy_order = self.buy_order.lock().unwrap();
        let mut sell_order = self.sell_order.lock().unwrap();

        println!("Buy order size {:?}", buy_order.len());
        println!("Sell order size {:?}", sell_order.len());
        while !buy_order.is_empty() && !sell_order.is_empty() {
            let sell = sell_order.peek().unwrap();
            let buy = buy_order.peek().unwrap();
            if buy.price() >= sell.price() {
                let sell = sell_order.pop().unwrap();
                let buy = buy_order.pop().unwrap();
                let amount = min(sell.amount(), buy.amount());
                p.close = (amount as f32 * buy.price() + p.volume as f32 * p.close)
                    / (amount + p.volume) as f32;
                p.borrow_mut().volume += amount;
                p.borrow_mut().high = f32::max(p.high, buy.price());
                p.borrow_mut().low = f32::min(p.low, buy.price());
                println!("Executed {:?} of stocks", amount);
                if amount != buy.amount() {
                    buy_order.push(Transaction::buy(
                        buy.symbol().to_string(),
                        buy.price(),
                        buy.amount() - amount,
                    ))
                }
                if amount != sell.amount() {
                    sell_order.push(Transaction::sell(
                        buy.symbol().to_string(),
                        buy.price(),
                        buy.amount() - amount,
                    ))
                }
            } else {
                break;
            }
        }
    }
}

#[test]
pub fn test_order_execution() {
    let buy_order = Arc::new(Mutex::new(BinaryHeap::new()));
    let sell_order = Arc::new(Mutex::new(BinaryHeap::new()));
    let buy = Transaction::buy("NVDA".to_string(), 500.0, 1000);
    let sell = Transaction::sell("NVDA".to_string(), 500.0, 1000);
    buy_order.lock().unwrap().push(buy);
    sell_order.lock().unwrap().push(sell);
    let prv_point = Point::new(400.0, 400.0, 400.0, 400.0, 1000);
    let arc_prv_point = Arc::new(Mutex::new(prv_point));
    OrderBook::new(
        buy_order.clone(),
        sell_order.clone(),
        Arc::new(RwLock::new(LinkedList::new())),
        Arc::clone(&arc_prv_point),
    )
    .execute();
    assert_eq!(arc_prv_point.lock().unwrap().high, 500.0);
    assert_eq!(arc_prv_point.lock().unwrap().close, 450.0);
    assert_eq!(arc_prv_point.lock().unwrap().low, 400.0);
    assert_eq!(arc_prv_point.lock().unwrap().volume, 2000);
    assert!(buy_order.lock().unwrap().is_empty());
    assert!(sell_order.lock().unwrap().is_empty());
}

#[test]
pub fn test_order_not_executed() {
    let buy_order = Arc::new(Mutex::new(BinaryHeap::new()));
    let sell_order = Arc::new(Mutex::new(BinaryHeap::new()));
    let buy = Transaction::buy("NVDA".to_string(), 500.0, 1000);
    let sell = Transaction::sell("NVDA".to_string(), 1000.0, 1000);
    buy_order.lock().unwrap().push(buy);
    sell_order.lock().unwrap().push(sell);
    let now = Arc::new(Mutex::new(Point::new(400.0, 400.0, 400.0, 400.0, 1000)));
    OrderBook::new(
        buy_order.clone(),
        sell_order.clone(),
        Arc::new(RwLock::new(LinkedList::new())),
        now.clone(),
    )
    .execute();
    assert_eq!(now.lock().unwrap().volume, 1000);
    assert!(buy_order.lock().unwrap().len() == 1);
    assert!(sell_order.lock().unwrap().len() == 1);
}
