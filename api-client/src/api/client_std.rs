use alloc::string::String;
use crate::types::Arrival;

pub struct CarrisClient {
    base_url: String,
    client: reqwest::Client,
}

impl CarrisClient {
    pub fn new() -> Self {
        Self {
            base_url: "https://api.carrismetropolitana.pt/v2".to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn arrivals_by_stop(&self, stop: &str) -> Result<Vec<Arrival>, reqwest::Error> {
        let url = format!("{}/arrivals/by_stop/{}", self.base_url, stop);
        self.client.get(url).send().await?.json::<Vec<Arrival>>().await
    }

    pub async fn get_stop_id(&self, stop: &str) -> Result<Option<String>, reqwest::Error> {
        let url = format!("{}/arrivals/by_stop/{}", self.base_url, stop);
        self.client.get(url).send().await?.json::<Option<String>>().await
    }
}