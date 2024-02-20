use serde::{Deserialize, Serialize};
use crate::structs::TimeSeries;

#[derive(Deserialize, Serialize)]
pub struct Stock {
    name: String,
    symbol: String,
    cur_price: f32,
    time_series: TimeSeries,
}

impl Stock {
    pub fn new(name: String, symbol: String, cur_price: f32, time_series: TimeSeries) -> Self {
        Self { name, symbol, cur_price, time_series }
    }
}