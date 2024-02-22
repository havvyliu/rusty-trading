use crate::structs::operation::Operation;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Transaction {
    symbol: String,
    price: f32,
    amount: u32,
    operation: Operation,
}

impl Ord for Transaction {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.price.total_cmp(&other.price)
    }
}

impl PartialOrd for Transaction {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.price.total_cmp(&other.price))
    }
}

impl PartialEq for Transaction {
    fn eq(&self, other: &Self) -> bool {
        self.price().eq(&other.price)
    }
}

impl Eq for Transaction {}

impl Transaction {
    pub fn buy(symbol: String, price: f32, amount: u32) -> Self {
        Self {
            symbol,
            price,
            amount,
            operation: Operation::Buy,
        }
    }

    pub fn sell(symbol: String, price: f32, amount: u32) -> Self {
        Self {
            symbol,
            price,
            amount,
            operation: Operation::Sell,
        }
    }

    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    pub fn price(&self) -> f32 {
        self.price
    }

    pub fn amount(&self) -> u32 {
        self.amount
    }

    pub fn operation(&self) -> &Operation {
        &self.operation
    }
}
