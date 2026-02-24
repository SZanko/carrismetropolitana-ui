mod config;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use slint::{Color, Image, ModelRc, SharedString, VecModel};
use carris_api::api::CarrisClient;
use carris_api::types::{Arrival, CarrisAPI, Stop};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

slint::include_modules!();

static API_CLIENT: OnceLock<CarrisClient> = OnceLock::new();

pub fn api_client() -> &'static CarrisClient {
    API_CLIENT.get_or_init(|| CarrisClient::new())
}

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


fn stops_to_models(stops: Vec<Stop>) -> (ModelRc<ListItem>, ModelRc<SharedString>) {
    let mut items = Vec::with_capacity(stops.len());
    let mut ids = Vec::with_capacity(stops.len());

    for s in stops {
        ids.push(SharedString::from(s.id.clone()));

        items.push(ListItem {
            text: s.long_name.into(),          // what user sees
            supporting_text: s.id.into(), // optional (e.g. municipality)
            avatar_icon: Image::default(),
            avatar_text: SharedString::new(),
            avatar_background: Color::from_argb_u8(0, 0, 0, 0),
            avatar_foreground: Color::from_argb_u8(0, 0, 0, 0),
            action_button_icon: Image::default(),
        });
    }

    (ModelRc::new(VecModel::from(items)), ModelRc::new(VecModel::from(ids)))
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn main() {
    let ui = ui();

    let lookup: Arc<Mutex<HashMap<String, String>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let lookup_for_task = lookup.clone();

    let ui_handle_stops = ui.clone_strong();

    slint::spawn_local(async_compat::Compat::new(async move {
        match config::get_all_stops_cached().await {
            Ok(stops) => {
                println!("Stops: {:?}", stops.len());
                let lookup_stops = stops.clone();
                let (bus_stations, bus_station_ids) = stops_to_models(stops);
                ui_handle_stops.set_bus_stations(bus_stations);

                let mut map = HashMap::with_capacity(lookup_stops.len());
                for s in lookup_stops {
                    map.insert(s.long_name, s.id);
                }
                *lookup_for_task.lock().unwrap() = map;
            }
            Err(e) => {
                eprintln!("Failed to load stops: {e}");
                ui_handle_stops.set_busstation_label("Cannot load all stops".into())
            }
        }

    })).unwrap();

    let ui_for_cb = ui.clone_strong();
    let lookup_for_cb = lookup.clone();
    ui.on_bus_station_selected(move |search_text: SharedString| {


        let name = search_text.to_string();

        let stop_id = {
            let map = lookup_for_cb.lock().unwrap();
            map.get(&name).cloned()
        };

        let Some(stop_id) = stop_id else {
            eprintln!("No stop id found for selection: {name}");
            return;
        };

        eprintln!("Selected stop: {stop_id}");
        let ui_for_task = ui_for_cb.clone_strong();
        slint::spawn_local(async_compat::Compat::new(async move {
            match api_client().arrivals_by_stop(&stop_id).await {
                Ok(arrivals) => {
                    let bus_arrivals: Vec<BusArrival> =
                        arrivals.into_iter().map(BusArrival::from).collect();
                    ui_for_task.set_next_busses(ModelRc::new(VecModel::from(bus_arrivals)));
                }
                Err(e) => eprintln!("Failed to load arrivals for {stop_id}: {e}"),
            }
        })).unwrap();
    });


    ui.on_searchbar_bus_station_clicked(move |index| {
        println!("Search for {index}");
    });


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


    let ui_handle_busses = ui.clone_strong();

    slint::spawn_local(async_compat::Compat::new(async move {

        println!("Get Bus Data for home STOP");
        // TODO don't hard code this
        let result = api_client().arrivals_by_stop("020387").await.unwrap();

        let bus_arrivals: Vec<BusArrival> = result.into_iter().map(BusArrival::from).collect();
        println!("Length of the content is: {}", bus_arrivals.len());
        let model = ModelRc::new(VecModel::from(bus_arrivals));
        ui_handle_busses.set_next_busses(model);
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
