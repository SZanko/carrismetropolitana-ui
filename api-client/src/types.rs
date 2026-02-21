use serde::{Deserialize, Deserializer};
use alloc::string::String;

#[derive(Debug, Deserialize)]
pub struct Arrival {
    pub estimated_arrival_unix: Option<i64>,
    pub observed_arrival_unix: Option<i64>,
    pub scheduled_arrival_unix: Option<i64>,

    #[serde(deserialize_with = "de_i16_from_string")]
    pub line_id: i16,

    pub headsign: String,
    pub scheduled_arrival: Option<String>,
}

fn de_i16_from_string<'de, D>(deserializer: D) -> Result<i16, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse::<i16>().map_err(serde::de::Error::custom)
}

pub fn best_arrival_unix(a: &Arrival) -> Option<i64> {
    a.estimated_arrival_unix
        .or(a.observed_arrival_unix)
        .or(a.scheduled_arrival_unix)
}