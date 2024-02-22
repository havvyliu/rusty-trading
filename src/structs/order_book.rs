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
                p.borrow_mut().low = f32::max(p.low, buy.price());
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
