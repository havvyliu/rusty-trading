use std::time::SystemTime;


pub struct Block {
    open: u32,
    high: u32,
    low: u32,
    close: u32,
    volume: u32,
}

impl Block {
    pub fn new(open: u32, high: u32, low: u32, close: u32, volume: u32) -> Self {
        Self {open, high, low, close, volume}
    }
}

pub struct TimeSeries {
    time_range_unit: TimeRange,
    start: SystemTime,
    end: SystemTime,
    data: Vec<Block>,
}

impl TimeSeries {
    pub fn new(time_range_unit: TimeRange, start: SystemTime, end: SystemTime, data: Vec<Block>) -> Self {
        Self {time_range_unit, start, end, data}
    }
}


pub enum TimeRange {
    Second,
    Minute,
    Hour,
    Daily,
    Montly,
}