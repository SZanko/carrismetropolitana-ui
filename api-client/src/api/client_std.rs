use alloc::string::String;
use crate::types::{Arrival, CarrisAPI, Stop};

pub struct CarrisClient {
    base_url: String,
    client: reqwest::Client,
}

impl CarrisAPI for CarrisClient {
    type Error = reqwest::Error;

    fn new() -> Self {
        Self {
            base_url: "https://api.carrismetropolitana.pt/v2".to_string(),
            client: reqwest::Client::new(),
        }
    }

    fn new_with_base_url(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_owned(),
            client: reqwest::Client::new(),
        }
    }

    async fn arrivals_by_stop(&self, stop: &str) -> Result<Vec<Arrival>, reqwest::Error> {
        let url = format!("{}/arrivals/by_stop/{}", self.base_url, stop);
        self.client.get(url).send().await?.json::<Vec<Arrival>>().await
    }


    async fn get_all_stops(&self) -> Result<Vec<Stop>, reqwest::Error> {
        let url = format!("{}/stops", self.base_url);
        self.client.get(url).send().await?.json::<Vec<Stop>>().await
    }
}