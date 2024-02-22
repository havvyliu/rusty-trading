use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
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

    pub fn borrow(&self) -> &Self {
        self
    }

    // pub fn open(&self) -> u32 {
    //     self.open
    // }

    // pub fn high(&self) -> u32 {
    //     self.high
    // }

    // pub fn low(&self) -> u32 {
    //     self.low
    // }
    // pub fn close(&self) -> u32 {
    //     self.close
    // }
    // pub fn volume(&self) -> u32 {
    //     self.volume
    // }
}
