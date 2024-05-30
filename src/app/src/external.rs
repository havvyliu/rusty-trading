use std::{collections::HashMap, fmt};

use chrono::{DateTime, NaiveDateTime, Utc};
use rusty_trading_lib::structs::Point;
use serde::{de::{MapAccess, Visitor}, Deserialize, Deserializer};

/*
{
  "Meta Data": {
    "1. Information": "Intraday (5min) open, high, low, close prices and volume",
    "2. Symbol": "NVDA",
    "3. Last Refreshed": "2024-05-28 19:55:00",
    "4. Interval": "5min",
    "5. Output Size": "Compact",
    "6. Time Zone": "US/Eastern"
  },
  "Time Series (5min)": {
    "2024-05-28 19:55:00": {
      "1. open": "1149.7200",
      "2. high": "1150.0000",
      "3. low": "1148.7300",
      "4. close": "1149.9900",
      "5. volume": "42000"
    },
    ...
*/
#[derive(Deserialize, Debug)]
pub struct IntradayStock {
    #[serde(alias = "Meta Data")]
    meta_data: MetaData,
    #[serde(
        alias = "Time Series (5min)",
    )]
    time_series: TimePointMap,
}


impl IntradayStock {
    pub fn get_points_map(self: &Self) -> &HashMap<DateTime<Utc>, Point> {
        &self.time_series.map
    }
}

#[derive(Deserialize, Debug)]
pub struct TimePointMap {
    #[serde(
        flatten,
        deserialize_with = "deserialize_hash_map",
    )]
    map: HashMap<DateTime<Utc>, Point>,
}

#[derive(Deserialize, Debug)]
pub struct CandleStick {
    #[serde(alias = "1. open")]
    open: String,
    #[serde(alias = "2. high")]
    high: String,
    #[serde(alias = "3. low")]
    low: String,
    #[serde(alias = "4. close")]
    close: String,
    #[serde(alias = "5. volume")]
    volume: String,
}

#[derive(Deserialize, Debug)]
pub struct MetaData {
    #[serde(alias = "1. Information")]
    information: String,
    #[serde(alias = "2. Symbol")]
    symbol: String,
    #[serde(alias = "3. Last Refreshed")]
    #[serde(deserialize_with = "deserialize_date_time")]
    last_refreshed: DateTime<Utc>,
    #[serde(alias = "4. Interval")]
    interval: String,
    #[serde(alias = "5. Output Size")]
    output_size: String,
    #[serde(alias = "6. Time Zone")]
    time_zone: String,
}

pub const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

fn deserialize_hash_map<'de, D>(deserializer: D) -> Result<HashMap<DateTime<Utc>, Point>, D::Error>
where
    D: Deserializer<'de>,
{
    struct MapVisitor;

    impl<'de> Visitor<'de> for MapVisitor {
        type Value = HashMap<DateTime<Utc>, Point>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("map should contain timestamp and candle stick data")
        }

        fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut result = HashMap::new();
            while let Some((key, candle_stick)) = map.next_entry::<String, CandleStick>()? {
                let dt = NaiveDateTime::parse_from_str(&key, FORMAT).map_err(serde::de::Error::custom)?;
                let utc_dt = DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc);
                let point = Point::new(
                    candle_stick.open.parse::<f32>().unwrap(),
                    candle_stick.high.parse::<f32>().unwrap(),
                    candle_stick.low.parse::<f32>().unwrap(),
                    candle_stick.close.parse::<f32>().unwrap(),
                    candle_stick.volume.parse::<u32>().unwrap());
                result.insert(utc_dt, point);
            }
            Ok(result)
        }
    }

    deserializer.deserialize_map(MapVisitor)
}

// The signature of a deserialize_with function must follow the pattern:
//
//    fn deserialize<'de, D>(D) -> Result<T, D::Error>
//    where
//        D: Deserializer<'de>
//
// although it may also be generic over the output types T.
pub fn deserialize_date_time<'de, D>(
    deserializer: D,
) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let dt = NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)?;
    Ok(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
}


#[test]
pub fn test_str_gets_deserialized_properly() {
    let json_str = r#"
    {
        "Meta Data": {
            "1. Information": "Intraday (5min) open, high, low, close prices and volume",
            "2. Symbol": "NVDA",
            "3. Last Refreshed": "2024-05-28 19:55:00",
            "4. Interval": "5min",
            "5. Output Size": "Compact",
            "6. Time Zone": "US/Eastern"
        },
        "Time Series (5min)": {
            "2024-05-28 19:55:00": {
                "1. open": "1149.7200",
                "2. high": "1150.0000",
                "3. low": "1148.7300",
                "4. close": "1149.9900",
                "5. volume": "42000"
            },
            "2024-05-28 19:50:00": {
                "1. open": "1149.0400",
                "2. high": "1149.9800",
                "3. low": "1149.0100",
                "4. close": "1149.7500",
                "5. volume": "30271"
            },
            "2024-05-28 19:45:00": {
                "1. open": "1148.9900",
                "2. high": "1149.3300",
                "3. low": "1148.5000",
                "4. close": "1149.1050",
                "5. volume": "15594"
            }
        }
    }
    "#;
    let intraday_stock: IntradayStock = serde_json::from_str(&json_str).unwrap();
    assert_eq!(intraday_stock.meta_data.symbol, "NVDA");
    assert_eq!(intraday_stock.time_series.map.keys().len(), 3);
    let str = "2024-05-28 19:45:00";
    let dt = NaiveDateTime::parse_from_str(&str, FORMAT).unwrap();
    let utc = DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc);
    assert_eq!(intraday_stock.time_series.map.get(&utc).unwrap().open, 1148.99 as f32);
    assert_eq!(intraday_stock.time_series.map.get(&utc).unwrap().volume, 15594 as u32);
}
