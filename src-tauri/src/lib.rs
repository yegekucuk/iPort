pub mod actions;
pub mod classifier;
pub mod commands;
pub mod enrichers;
pub mod models;
pub mod monitor;
pub mod scanner;

use tauri::Manager;

use crate::monitor::{start_polling, AppState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .setup(|app| {
            let state = app.state::<AppState>().inner().clone();
            start_polling(app.handle().clone(), state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_port_snapshot,
            commands::refresh_ports,
            commands::get_system_theme,
            commands::stop_process,
            commands::kill_process,
            commands::open_in_browser,
            commands::reveal_in_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running iPort application");
}
