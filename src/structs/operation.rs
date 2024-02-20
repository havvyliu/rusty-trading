use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
pub enum Operation {
    Buy,
    Sell,
}