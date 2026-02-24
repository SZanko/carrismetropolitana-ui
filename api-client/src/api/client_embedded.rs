use core::fmt::Write;
use embedded_io_async::Read;
use reqwless::{client::HttpClient, request::Method};

use crate::types::Arrival;

pub struct CarrisClient<'a, TCP, DNS>
where
    TCP: embedded_nal_async::TcpConnect,
    DNS: embedded_nal_async::Dns,
{
    http: HttpClient<'a, TCP, DNS>,
    rx_buf: &'a mut [u8],
    body_buf: &'a mut [u8],
}

#[derive(Debug)]
pub enum Error<E> {
    Http(E),
    Json(serde_json::Error),
    TooLarge,
}

impl<'a, TCP, DNS> CarrisClient<'a, TCP, DNS>
where
    TCP: embedded_nal_async::TcpConnect,
    DNS: embedded_nal_async::Dns,
{
    pub fn new(http: HttpClient<'a, TCP, DNS>, rx_buf: &'a mut [u8], body_buf: &'a mut [u8]) -> Self {
        Self { http, rx_buf, body_buf }
    }

    pub async fn arrivals_by_stop(
        &mut self,
        stop_id: &str,
    ) -> Result<alloc::vec::Vec<Arrival>, Error<reqwless::Error>> {
        let mut url: heapless::String<128> = heapless::String::new();
        write!(
            &mut url,
            "https://api.carrismetropolitana.pt/v2/arrivals/by_stop/{}",
            stop_id
        )
            .map_err(|_| Error::TooLarge)?;

        let mut req = self
            .http
            .request(Method::GET, url.as_str())
            .await
            .map_err(Error::Http)?;

        let response = req.send(self.rx_buf).await.map_err(Error::Http)?;

        let mut total = 0usize;
        let mut reader = response.body().reader();

        loop {
            if total >= self.body_buf.len() {
                return Err(Error::TooLarge);
            }
            let n = reader
                .read(&mut self.body_buf[total..])
                .await
                .map_err(Error::Http)?;
            if n == 0 {
                break;
            }
            total += n;
        }

        let arrivals: alloc::vec::Vec<Arrival> =
            serde_json::from_slice(&self.body_buf[..total]).map_err(Error::Json)?;

        Ok(arrivals)
    }
}