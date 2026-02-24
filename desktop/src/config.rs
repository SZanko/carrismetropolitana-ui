use std::fs;
use std::io::Write;
use std::fs::File;
use std::sync::OnceLock;
use xdg::BaseDirectories;
use carris_api::types::{CarrisAPI, Stop};
use crate::api_client;

static XDG_DIRS: OnceLock<BaseDirectories> = OnceLock::new();

fn xdg_dirs() -> &'static BaseDirectories {
    XDG_DIRS.get_or_init(|| {
        BaseDirectories::with_prefix("carris-ui")
    })
}

pub async fn ensure_stops_cached_with<A>(
    xdg: &BaseDirectories,
    api: &A,
) -> anyhow::Result<()>
where
    A: CarrisAPI,
    A::Error: std::error::Error + Send + Sync + 'static,
{
    if xdg.find_cache_file("all_stops.json").is_some() {
        return Ok(());
    }

    println!("Check if stops file exists and if not download it");
    let stops = api.get_all_stops().await?;
    println!("{:?}", stops);
    let json_stops = serde_json::to_vec_pretty(&stops)?;
    let path = xdg.place_cache_file("all_stops.json")?;
    fs::write(path, json_stops)?;

    Ok(())
}

pub async fn ensure_stops_cached_if_not_cached_download() -> anyhow::Result<()> {
    ensure_stops_cached_with(xdg_dirs(), api_client()).await
}


pub async fn get_all_stops_cached() -> anyhow::Result<Vec<Stop>> {
    ensure_stops_cached_if_not_cached_download().await?;

    let path = xdg_dirs()
        .find_cache_file("all_stops.json")
        .ok_or_else(|| anyhow::anyhow!("cache file all_stops.json still missing after ensure"))?;

    let bytes = fs::read(path)?;
    let stops: Vec<Stop> = serde_json::from_slice(&bytes)?;

    Ok(stops)
}

pub fn load_config_from_disk() {

}

pub fn save_config() -> std::io::Result<()> {
    let config_path = xdg_dirs()
        .place_config_file("config.toml")
        .expect("cannot create configuration directory");

    let mut config_file = File::create(config_path)?;
    write!(&mut config_file, "configured = 1")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use xdg::BaseDirectories;
    use core::future::Future;
    use carris_api::types::Arrival;

    struct FakeApi {
        stops: Vec<Stop>,
        calls: std::sync::atomic::AtomicUsize,
    }

    impl CarrisAPI for FakeApi {
        type Error = anyhow::Error;

        fn new() -> Self {
            Self {
                stops: vec![],
                calls: std::sync::atomic::AtomicUsize::new(0),
            }
        }

        fn new_with_base_url(_base_url: &str) -> Self {
            Self::new()
        }

        fn arrivals_by_stop<'a>(
            &'a self,
            _stop: &'a str,
        ) -> impl Future<Output = Result<Vec<Arrival>, Self::Error>> + 'a {
            async move { Ok(vec![]) }
        }

        fn get_all_stops<'a>(
            &'a self,
        ) -> impl Future<Output = Result<Vec<Stop>, Self::Error>> + 'a {
            async move {
                self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(self.stops.clone())
            }
        }
    }

}