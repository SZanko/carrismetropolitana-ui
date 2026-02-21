use std::io::Write;
use std::fs::File;
use std::sync::OnceLock;
use xdg::BaseDirectories;

static XDG_DIRS: OnceLock<BaseDirectories> = OnceLock::new();

fn xdg_dirs() -> &'static BaseDirectories {
    XDG_DIRS.get_or_init(|| {
        BaseDirectories::with_prefix("carris-ui")
    })
}

pub fn check_if_stops_exists_on_disk() {
    //xdg_dirs().find_cache_file("all_stops.json").unwrap_or_else(|| {
    //
    //})
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