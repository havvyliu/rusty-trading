use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub enum TimeRange {
    Second,
    Minute,
    Hour,
    Day,
    Month,
}
