use serde::{Deserialize, Deserializer, Serialize};
use alloc::string::String;
use serde_json::Value;

// https://transform.tools/json-to-rust-serde
#[derive(Default, Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Arrival {
    pub estimated_arrival_unix: Option<i64>,
    pub observed_arrival_unix: Option<i64>,
    pub scheduled_arrival_unix: Option<i64>,

    #[serde(deserialize_with = "de_i16_from_string")]
    pub line_id: i16,

    pub headsign: String,
    pub scheduled_arrival: Option<String>,
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stop {
    #[serde(rename = "district_id")]
    pub district_id: Value,
    pub facilities: Vec<Value>,
    pub id: String,
    pub lat: f64,
    #[serde(rename = "line_ids")]
    pub line_ids: Vec<String>,
    pub lon: f64,
    #[serde(rename = "long_name")]
    pub long_name: String,
    #[serde(rename = "municipality_id")]
    pub municipality_id: Value,
    #[serde(rename = "pattern_ids")]
    pub pattern_ids: Vec<String>,
    #[serde(rename = "region_id")]
    pub region_id: Value,
    #[serde(rename = "route_ids")]
    pub route_ids: Vec<String>,
    #[serde(rename = "short_name")]
    pub short_name: Value,
    #[serde(rename = "tts_name")]
    pub tts_name: String,
    #[serde(rename = "wheelchair_boarding")]
    pub wheelchair_boarding: bool,
}

fn de_i16_from_string<'de, D>(deserializer: D) -> Result<i16, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse::<i16>().map_err(serde::de::Error::custom)
}

pub trait CarrisAPI {
    type Error;

    fn new()->Self;
    fn new_with_base_url(base_url: &str) -> Self;

    fn arrivals_by_stop<'a>(
        &'a self,
        stop: &'a str,
    ) -> impl Future<Output = Result<Vec<Arrival>, Self::Error>> + 'a;

    fn get_all_stops<'a>(
        &'a self,
    ) -> impl Future<Output = Result<Vec<Stop>, Self::Error>> + 'a;
}

pub fn best_arrival_unix(a: &Arrival) -> Option<i64> {
    a.estimated_arrival_unix
        .or(a.observed_arrival_unix)
        .or(a.scheduled_arrival_unix)
}