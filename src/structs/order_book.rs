use std::{
    borrow::BorrowMut,
    cmp::min,
    collections::{BinaryHeap, LinkedList},
    sync::{Arc, Mutex},
};

use crate::{Point, Transaction};

#[derive(Clone)]
pub struct OrderBook {
    buy_order: Arc<Mutex<BinaryHeap<Transaction>>>,
    sell_order: Arc<Mutex<BinaryHeap<Transaction>>>,
    points_queue: Arc<Mutex<LinkedList<Point>>>,
}

impl OrderBook {
    pub fn new(
        buy_order: Arc<Mutex<BinaryHeap<Transaction>>>,
        sell_order: Arc<Mutex<BinaryHeap<Transaction>>>,
        points_queue: Arc<Mutex<LinkedList<Point>>>,
    ) -> Self {
        Self {
            buy_order,
            sell_order,
            points_queue,
        }
    }

    pub fn add_buy_order(&self, buy_order: Transaction) {
        self.buy_order.lock().unwrap().push(buy_order);
    }

    pub fn add_sell_order(&self, sell_order: Transaction) {
        self.sell_order.lock().unwrap().push(sell_order);
    }

    pub fn execute(&self) -> Point {
        let mut p = self
            .points_queue
            .lock()
            .unwrap()
            .pop_front()
            .or(Some(Point::blank()))
            .unwrap();
        let mut buy_order = self.buy_order.lock().unwrap();
        let mut sell_order = self.sell_order.lock().unwrap();

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
        p
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
    let mut q = LinkedList::new();
    q.push_front(prv_point);
    let now = OrderBook::new(
        buy_order.clone(),
        sell_order.clone(),
        Arc::new(Mutex::new(q)),
    )
    .execute();
    assert_eq!(now.high, 500.0);
    assert_eq!(now.close, 450.0);
    assert_eq!(now.low, 400.0);
    assert_eq!(now.volume, 2000);
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
    let prv_point = Point::new(400.0, 400.0, 400.0, 400.0, 1000);
    let mut q = LinkedList::new();
    q.push_front(prv_point);
    let now = OrderBook::new(
        buy_order.clone(),
        sell_order.clone(),
        Arc::new(Mutex::new(q)),
    )
    .execute();
    assert_eq!(now.volume, 1000);
    assert!(buy_order.lock().unwrap().len() == 1);
    assert!(sell_order.lock().unwrap().len() == 1);
}
