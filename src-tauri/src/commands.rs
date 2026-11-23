use tauri::{AppHandle, State};

use crate::actions;
use crate::models::{ActionResult, PortSnapshot, SystemThemeInfo};
use crate::monitor::{refresh_now, AppState};

#[tauri::command]
pub async fn get_port_snapshot(state: State<'_, AppState>) -> Result<PortSnapshot, String> {
    let latest = state.latest.read().await.clone();
    if latest.timestamp_ms > 0 {
        return Ok(latest);
    }
    Ok(refresh_now(&state).await)
}

#[tauri::command]
pub async fn refresh_ports(state: State<'_, AppState>) -> Result<PortSnapshot, String> {
    Ok(refresh_now(&state).await)
}

#[tauri::command]
pub fn get_system_theme() -> SystemThemeInfo {
    actions::read_system_theme()
}

#[tauri::command]
pub fn stop_process(pid: u32) -> ActionResult {
    actions::stop_process(pid)
}

#[tauri::command]
pub fn kill_process(pid: u32) -> ActionResult {
    actions::kill_process(pid)
}

#[tauri::command]
pub fn open_in_browser(
    app: AppHandle,
    port: u16,
    bind_address: String,
    use_https: Option<bool>,
) -> ActionResult {
    actions::open_in_browser(&app, port, &bind_address, use_https)
}

#[tauri::command]
pub fn reveal_in_folder(app: AppHandle, path: String) -> ActionResult {
    actions::reveal_in_folder(&app, &path)
}

#[tauri::command]
pub fn open_in_ide(path: String, ide: String) -> ActionResult {
    actions::open_in_ide(&path, &ide)
}
