use serde::{Deserialize, Serialize};
use crate::structs::operation::Operation;
use crate::structs::stock::Stock;

#[derive(Serialize, Deserialize)]
pub struct Transaction<'a> {
    stock: &'a Stock,
    price: f32,
    amount: i32,
    operation: Operation,
}


impl Transaction {
    pub fn buy(stock: &mut Stock, price: f32, amount: i32) -> Self {
        Self {stock, price, amount, operation: Operation::Buy }
    }

    pub fn sell(stock: &mut Stock, price: f32, amount: i32) -> Self {
        Self {stock, price, amount, operation: Operation::Sell }
    }
}
