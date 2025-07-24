use mitch::{mitch::Mitch, Mitches};
use tauri::{async_runtime::spawn, Manager as _, State};

pub mod errors;
pub mod mitch;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
async fn get_mitches<'a>(mitches: State<'a, Mitches>) -> Result<Vec<Mitch>, ()> {
    let mitches = mitches.inner.lock().await.to_vec();
    println!("{:?}", mitches);
    Ok(mitches)
}
//
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
async fn get_mitch_details<'a>(id: usize, mitches: State<'a, Mitches>) -> Result<Mitch, ()> {
    Ok(mitches.inner.lock().await[id].clone())
}

#[tauri::command]
async fn connect<'a>(id: usize, mitches: State<'a, Mitches>) -> Result<(), String> {
    let mut l = mitches.inner.lock().await;
    l[id].connect().await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn disconnect<'a>(id: usize, mitches: State<'a, Mitches>) -> Result<(), errors::Error> {
    let mut l = mitches.inner.lock().await;
    l[id]
        .disconnect()
        .await
        .map_err(|_| errors::Error::Disconnect)?;
    Ok(())
}

#[tauri::command]
async fn start_recording<'a>(id: usize, mitches: State<'a, Mitches>) -> Result<(), ()> {
    mitches.inner.lock().await[id]
        .start_recording()
        .await
        .unwrap();
    Ok(())
}

#[tauri::command]
async fn stop_recording<'a>(id: usize, mitches: State<'a, Mitches>) -> Result<(), ()> {
    mitches.inner.lock().await[id]
        .stop_recording()
        .await
        .unwrap();
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default().plugin(tauri_plugin_opener::init());
    builder
        .setup(|app| {
            app.manage(mitch::Mitches::default());
            let app_handle = app.handle().clone();
            spawn(mitch::discover(app_handle));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_mitches,
            connect,
            disconnect,
            get_mitch_details,
            start_recording,
            stop_recording
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
