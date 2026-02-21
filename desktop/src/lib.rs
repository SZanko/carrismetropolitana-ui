use std::time::Instant;
use slint::{ModelRc, VecModel};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

slint::include_modules!();

fn ui() -> MainWindow {
    MainWindow::new().unwrap()
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn main() {
    let ui = ui();

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

    let model = ModelRc::new(VecModel::from(dummy_busses));
    ui.set_next_busses(model);
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
