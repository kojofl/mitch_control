use std::{collections::HashSet, process::exit};

use btleplug::{
    api::{Central as _, CentralEvent, CentralState, Manager as _, Peripheral as _, ScanFilter},
    platform::{Adapter, Manager, Peripheral},
};
use futures::StreamExt as _;
use mitch::Mitch;
use serde::Serialize;
use tauri::{async_runtime::Mutex, AppHandle, Emitter, Manager as _};
pub mod mitch;

#[derive(Default)]
pub struct Mitches {
    pub inner: Mutex<Vec<mitch::Mitch>>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MitchDiscovered<'a> {
    pub name: &'a str,
}

async fn get_central(manager: &Manager) -> Adapter {
    let adapters = manager.adapters().await.unwrap();
    adapters.into_iter().next().unwrap()
}

pub(crate) async fn discover(app: AppHandle) {
    let manager = Manager::new().await.expect("valid bluetooth manager");

    let central = get_central(&manager).await;

    let central_state = central.adapter_state().await.unwrap();

    if central_state != CentralState::PoweredOn {
        println!("Bluetooth turned off");
        exit(1);
    }

    let mut events = central.events().await.unwrap();

    central.start_scan(ScanFilter::default()).await.unwrap();

    let mut discovered = HashSet::new();

    while let Some(event) = events.next().await {
        if let CentralEvent::DeviceDiscovered(id) = event {
            let peripheral = central.peripheral(&id).await.unwrap();
            let properties = peripheral.properties().await.unwrap();
            let name = properties
                .and_then(|p| p.local_name)
                .unwrap_or_default()
                .to_lowercase();
            if name.starts_with("mitch") {
                if discovered.insert(name.clone()) {
                    let mitches: tauri::State<'_, Mitches> = app.state();
                    println!("Discovered: {name}");
                    mitches
                        .inner
                        .lock()
                        .await
                        .push(Mitch::new(name.as_str(), peripheral));
                    app.emit(
                        "mitch-discovered",
                        MitchDiscovered {
                            name: name.as_ref(),
                        },
                    )
                    .unwrap();
                }
            }
        }
    }
}
