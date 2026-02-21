mod config;

use std::ptr::null;
use std::time::Instant;
use slint::{ModelRc, VecModel};
use carris_api::api::CarrisClient;
use carris_api::types::Arrival;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

slint::include_modules!();

fn ui() -> MainWindow {
    MainWindow::new().unwrap()
}

impl From<Arrival> for BusArrival {
    fn from(arrival: Arrival) -> Self {
        BusArrival {
            number: arrival.line_id as i32,

            arrival_time: arrival.scheduled_arrival.unwrap_or_else(|| "--:--".into()).into(),

            direction: arrival.headsign.into(),
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn main() {
    let mut api = CarrisClient::new();
    let ui = ui();

    slint::spawn_local(async_compat::Compat::new(async move {
        println!("Check if stops file exists and if not download it")

    })).unwrap();

    let dummy_busses = vec![
        BusArrival {
            number: 742,
            arrival_time: "10:05".into(),
            direction: "Monte".into(),
        },
        BusArrival {
            number: 69,
            arrival_time: "10:15".into(),
            direction: "Monte".into(),
        },
    ];

    let ui_handle = ui.clone_strong();


    slint::spawn_local(async_compat::Compat::new(async move {

        println!("Get Bus Data for home STOP");

        // TODO don't hard code this
        let result = api.arrivals_by_stop("020387").await.unwrap();

        let bus_arrivals: Vec<BusArrival> = result.into_iter().map(BusArrival::from).collect();
        println!("Length of the content is: {}", bus_arrivals.len());
        let model = ModelRc::new(VecModel::from(bus_arrivals));
        ui_handle.set_next_busses(model);
    })).expect("Cannot get Bus Data");

    //let model = ModelRc::new(VecModel::from(dummy_busses));
    //ui.set_next_busses(model);

    ui.run().unwrap();
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(android_app: slint::android::AndroidApp) {
    slint::android::init(android_app).unwrap();
    let ui = ui();
    MaterialWindowAdapter::get(&ui).set_disable_hover(true);
    ui.run().unwrap();
}
