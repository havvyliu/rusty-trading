use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::structs::Point;
use crate::structs::TimeRange;

#[derive(Deserialize, Serialize)]
pub struct TimeSeries {
    time_range_unit: TimeRange,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    data: Vec<Point>,
}

impl TimeSeries {
    pub fn new(time_range_unit: TimeRange, start: DateTime<Utc>, end: DateTime<Utc>, data: Vec<Point>) -> Self {
        Self {time_range_unit, start, end, data}
    }
}

