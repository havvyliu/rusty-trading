use std::{borrow::BorrowMut, cmp::min, collections::BinaryHeap};

use crate::{Point, Transaction};

struct OrderBook<'a> {
    buy_order: &'a mut BinaryHeap<Transaction>,
    sell_order: &'a mut BinaryHeap<Transaction>,
}

impl<'a> OrderBook<'a> {
    pub fn new(
        buy_order: &'a mut BinaryHeap<Transaction>,
        sell_order: &'a mut BinaryHeap<Transaction>,
    ) -> Self {
        Self {
            buy_order,
            sell_order,
        }
    }

    pub fn execute(&mut self, prv_point: Point) -> Point {
        let mut p = prv_point;

        while !self.sell_order.is_empty() && !self.buy_order.is_empty() {
            let sell = self.sell_order.peek().unwrap();
            let buy = self.buy_order.peek().unwrap();
            if buy.price() >= sell.price() {
                let sell = self.sell_order.pop().unwrap();
                let buy = self.buy_order.pop().unwrap();
                let amount = min(sell.amount(), buy.amount());
                p.borrow_mut().close = (amount as f32 * buy.price() + p.volume as f32 * p.close)
                    / (amount + p.volume) as f32;
                p.borrow_mut().volume += amount;
                p.borrow_mut().high = f32::max(p.high, buy.price());
                p.borrow_mut().low = f32::min(p.low, buy.price());
                if amount != buy.amount() {
                    self.buy_order.push(Transaction::buy(
                        buy.symbol().to_string(),
                        buy.price(),
                        buy.amount() - amount,
                    ))
                }
                if amount != sell.amount() {
                    self.sell_order.push(Transaction::sell(
                        buy.symbol().to_string(),
                        buy.price(),
                        buy.amount() - amount,
                    ))
                }
            }
        }
        p
    }
}

#[test]
pub fn test_order_execution() {
    let mut buy_order = BinaryHeap::new();
    let mut sell_order = BinaryHeap::new();
    let buy = Transaction::buy("NVDA".to_string(), 500.0, 1000);
    let sell = Transaction::sell("NVDA".to_string(), 500.0, 1000);
    buy_order.push(buy);
    sell_order.push(sell);
    let prv_point = Point::new(400.0, 400.0, 400.0, 400.0, 1000);
    let now = OrderBook::new(&mut buy_order, &mut sell_order).execute(prv_point);
    assert_eq!(now.high, 500.0);
    assert_eq!(now.close, 450.0);
    assert_eq!(now.low, 400.0);
    assert_eq!(now.volume, 2000);
    assert!(buy_order.is_empty());
    assert!(sell_order.is_empty());
}
