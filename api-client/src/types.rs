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
#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stop {
    #[serde(rename = "district_id")]
    pub district_id: String,
    pub facilities: Vec<String>,
    pub id: String,
    pub lat: f64,
    #[serde(rename = "line_ids")]
    pub line_ids: Vec<String>,
    #[serde(rename = "locality_id")]
    pub locality_id: String,
    pub lon: f64,
    #[serde(rename = "long_name")]
    pub long_name: String,
    #[serde(rename = "municipality_id")]
    pub municipality_id: String,
    #[serde(rename = "operational_status")]
    pub operational_status: String,
    #[serde(rename = "pattern_ids")]
    pub pattern_ids: Vec<String>,
    #[serde(rename = "region_id")]
    pub region_id: String,
    #[serde(rename = "route_ids")]
    pub route_ids: Vec<String>,
    #[serde(rename = "short_name")]
    pub short_name: String,
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

pub fn best_arrival_unix(a: &Arrival) -> Option<i64> {
    a.estimated_arrival_unix
        .or(a.observed_arrival_unix)
        .or(a.scheduled_arrival_unix)
}