use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Point {
    pub open: u32,
    high: u32,
    low: u32,
    close: u32,
    volume: u32,
}

impl Point {
    pub fn new(open: u32, high: u32, low: u32, close: u32, volume: u32) -> Self {
        Self {open, high, low, close, volume}
    }

    pub fn borrow(&self) -> &Self {
        self
    }
}