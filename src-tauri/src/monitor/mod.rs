use std::sync::{Arc, Mutex};
use std::time::Duration;

use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;

use crate::models::PortSnapshot;
use crate::scanner::PortScanner;

#[derive(Clone)]
pub struct AppState {
    pub scanner: Arc<Mutex<PortScanner>>,
    pub latest: Arc<RwLock<PortSnapshot>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            scanner: Arc::new(Mutex::new(PortScanner::new())),
            latest: Arc::new(RwLock::new(PortSnapshot::default())),
        }
    }

    pub fn scan_blocking_sync(&self) -> PortSnapshot {
        let mut scanner = self
            .scanner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        scanner.scan()
    }
}

pub async fn refresh_now(state: &AppState) -> PortSnapshot {
    let scanner = state.scanner.clone();
    let snapshot = tokio::task::spawn_blocking(move || {
        let mut guard = scanner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        guard.scan()
    })
    .await
    .unwrap_or_default();

    let mut latest = state.latest.write().await;
    *latest = snapshot.clone();
    snapshot
}

pub fn start_polling(app: AppHandle, state: AppState) {
    tauri::async_runtime::spawn(async move {
        let initial = refresh_now(&state).await;
        let _ = app.emit("ports:snapshot", &initial);

        let mut ticker = tokio::time::interval(Duration::from_secs(2));
        // Skip the immediate first tick since we just emitted the initial snapshot.
        ticker.tick().await;

        loop {
            ticker.tick().await;
            let snapshot = refresh_now(&state).await;
            let _ = app.emit("ports:snapshot", &snapshot);
        }
    });
}
