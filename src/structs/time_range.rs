use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub enum TimeRange {
    Second,
    FiveMinute,
    Minute,
    Hour,
    Day,
    Month,
}
