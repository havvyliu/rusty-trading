use std::f32::NAN;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct Point {
    pub open: f32,
    pub high: f32,
    pub low: f32,
    pub close: f32,
    pub volume: u32,
}

impl Point {
    pub fn new(open: f32, high: f32, low: f32, close: f32, volume: u32) -> Self {
        Self {
            open,
            high,
            low,
            close,
            volume,
        }
    }

    pub fn blank() -> Self {
        Self {
            open: NAN,
            high: 0.0,
            low: NAN,
            close: NAN,
            volume: 0,
        }
    }

    pub fn borrow(&self) -> &Self {
        self
    }
}
