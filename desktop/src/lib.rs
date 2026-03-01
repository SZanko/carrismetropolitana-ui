mod config;

use carris_api::api::CarrisClient;
use carris_api::types::{Arrival, CarrisAPI, Stop};
use log::log;
use slint::{Color, Image, ModelRc, SharedString, VecModel, Weak};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
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

            arrival_time: arrival
                .scheduled_arrival
                .unwrap_or_else(|| "--:--".into())
                .into(),

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
            text: s.long_name.into(),     // what user sees
            supporting_text: s.id.into(), // optional (e.g. municipality)
            avatar_icon: Image::default(),
            avatar_text: SharedString::new(),
            avatar_background: Color::from_argb_u8(0, 0, 0, 0),
            avatar_foreground: Color::from_argb_u8(0, 0, 0, 0),
            action_button_icon: Image::default(),
        });
    }

    (
        ModelRc::new(VecModel::from(items)),
        ModelRc::new(VecModel::from(ids)),
    )
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn main() {
    env_logger::init();
    let ui = ui();

    let lookup: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
    let lookup_for_task = lookup.clone();
    let lookup_for_stop = lookup.clone();
    let ui_handle_stops = ui.clone_strong();

    fill_searchbar_with_options(&ui, lookup_for_task);

    update_selected_bus_stop(&ui, &lookup);

    ui.on_searchbar_bus_station_clicked(move |index| {
        log::info!("Search for {index}");
    });

    let ui_searchbar_weak = ui.as_weak();
    let lookup_searchbar_cb = lookup.clone();

    ui.on_filter_searchbar_options(move |text: SharedString| {
        spawn_filter_search(ui_searchbar_weak.clone(), lookup_searchbar_cb.clone(), text);
    });


    set_bus_arrivals_for_station(&ui);

    //let model = ModelRc::new(VecModel::from(dummy_busses));
    //ui.set_next_busses(model);

    ui.run().unwrap();
}

fn spawn_filter_search(
    ui_weak: Weak<MainWindow>,
    lookup: Arc<Mutex<HashMap<String, String>>>,
    text: SharedString) {
    slint::spawn_local(async_compat::Compat::new(async move {
        if text.is_empty() {
            return;
        }

        let query = text.to_string();
        log::info!("(async) got text = {query}");

        let results = filter_search_string(&query, &lookup, 25);
        let mut filter_result: Vec<ListItem> = Vec::with_capacity(results.len());

        let map = lookup.lock().unwrap();

        for result in results {
            let key = result.clone();

            let supporting = map
                .get(key.as_str())
                .cloned()
                .unwrap_or_default();

            filter_result.push(ListItem {
                text: result.into(),
                supporting_text: supporting.into(),
                avatar_icon: Image::default(),
                avatar_text: SharedString::new(),
                avatar_background: Color::from_argb_u8(0, 0, 0, 0),
                avatar_foreground: Color::from_argb_u8(0, 0, 0, 0),
                action_button_icon: Image::default(),
            })
        }
        drop(map);

        if let Some(ui) = ui_weak.upgrade() {
            ui.set_bus_stations(ModelRc::new(VecModel::from(filter_result)));
        }
    })).expect("Cannot filter search options in searchbar");
}

fn filter_search_string(
    query: &str,
    lookup: &Arc<Mutex<HashMap<String, String>>>,
    limit: usize,
) -> Vec<SharedString> {
    let lquery = query.trim().to_lowercase();
    if lquery.is_empty() {
        return vec![];
    }

    let keys: Vec<String> = {
        let map = lookup.lock().unwrap();
        map.keys().cloned().collect()
    };

    let results = keys
        .into_iter()
        .filter(|name| name.to_lowercase().contains(&lquery))
        //.take(limit)
        .map(|name| SharedString::from(name))
        .collect();

    return results;
}

fn update_selected_bus_stop(ui: &MainWindow, lookup: &Arc<Mutex<HashMap<String, String>>>) {
    let ui_for_cb = ui.clone_strong();
    let lookup_for_cb = lookup.clone();
    ui.on_bus_station_selected(move |search_text: SharedString| {
        let name = search_text.to_string();

        let stop_id = {
            let map = lookup_for_cb.lock().unwrap();
            map.get(&name).cloned()
        };

        let Some(stop_id) = stop_id else {
            log::error!("No stop id found for selection: {name}");
            return;
        };

        log::info!("Selected stop: {stop_id}");
        let ui_for_task = ui_for_cb.clone_strong();
        slint::spawn_local(async_compat::Compat::new(async move {
            match api_client().arrivals_by_stop(&stop_id).await {
                Ok(arrivals) => {
                    let bus_arrivals: Vec<BusArrival> =
                        arrivals.into_iter().map(BusArrival::from).collect();
                    ui_for_task.set_next_busses(ModelRc::new(VecModel::from(bus_arrivals)));
                }
                Err(e) => log::error!("Failed to load arrivals for {stop_id}: {e}"),
            }
        }))
            .unwrap();
    });
}

fn set_bus_arrivals_for_station(ui: &MainWindow) {
    let ui_handle_busses = ui.clone_strong();

    slint::spawn_local(async_compat::Compat::new(async move {
        // TODO don't hard code this
        let bus_stop_id = "020387";
        log::info!("Getting bus data for {} id", bus_stop_id);
        let result = api_client().arrivals_by_stop(bus_stop_id).await.unwrap();

        let bus_arrivals: Vec<BusArrival> = result.into_iter().map(BusArrival::from).collect();
        log::info!("Length of the content is: {}", bus_arrivals.len());
        let model = ModelRc::new(VecModel::from(bus_arrivals));
        ui_handle_busses.set_next_busses(model);
    }))
        .expect("Cannot get Bus Data");
}

fn fill_searchbar_with_options(
    ui: &MainWindow,
    lookup_for_stop: Arc<Mutex<HashMap<String, String>>>,
) {
    let ui_handle_stops = ui.clone_strong();

    slint::spawn_local(async_compat::Compat::new(async move {
        match config::get_all_stops_cached().await {
            Ok(stops) => {
                log::info!("Stops: {:?}", stops.len());
                let lookup_stops = stops.clone();
                let (bus_stations, bus_station_ids) = stops_to_models(stops);
                ui_handle_stops.set_bus_stations(bus_stations);

                let mut map = HashMap::with_capacity(lookup_stops.len());
                for s in lookup_stops {
                    map.insert(s.long_name, s.id);
                }
                *lookup_for_stop.lock().unwrap() = map;
            }
            Err(e) => {
                log::error!("Failed to load stops: {e}");
                ui_handle_stops.set_busstation_label("Cannot load all stops".into())
            }
        }
    }))
        .unwrap();
}
//fn filter_search_results(input: &str, existing_bus_stops_original: Vec<>, ) -> ModelRc<ListItem> {
//
//}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(android_app: slint::android::AndroidApp) {
    slint::android::init(android_app).unwrap();
    let ui = ui();
    MaterialWindowAdapter::get(&ui).set_disable_hover(true);
    ui.run().unwrap();
}
