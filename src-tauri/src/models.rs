use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Tcp,
    Udp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    Local,
    Docker,
    Service,
    Tunnel,
    ThirdParty,
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppCategory {
    Dev,
    Docker,
    Service,
    Tunnel,
    ThirdParty,
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceManager {
    None,
    Pm2,
    Brew,
    Systemd,
    Launchd,
    WindowsService,
    Docker,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ManagedAction {
    Stop,
    ForceKill,
    Restart,
    OpenBrowser,
    CopyUrl,
    CopyPort,
    RevealFolder,
    OpenIde,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortEntry {
    pub id: String,
    pub port: u16,
    pub protocol: Protocol,
    pub bind_address: String,
    pub state: String,
    pub pid: Option<u32>,
    pub ppid: Option<u32>,
    pub process_name: String,
    pub exe_path: Option<String>,
    pub command_line: String,
    pub cwd: Option<String>,
    pub project_name: Option<String>,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub uptime_secs: Option<u64>,
    pub source_type: SourceType,
    pub category: AppCategory,
    pub is_core_os: bool,
    pub framework: Option<String>,
    pub runtime: Option<String>,
    pub service_manager: ServiceManager,
    pub available_actions: Vec<ManagedAction>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotCounts {
    pub total_visible: usize,
    pub dev: usize,
    pub docker: usize,
    pub services: usize,
    pub tunnels: usize,
    pub third_party: usize,
    pub system: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortSnapshot {
    pub timestamp_ms: u64,
    pub entries: Vec<PortEntry>,
    pub counts: SnapshotCounts,
}

impl Default for PortSnapshot {
    fn default() -> Self {
        Self {
            timestamp_ms: 0,
            entries: Vec::new(),
            counts: SnapshotCounts::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionResult {
    pub ok: bool,
    pub message: String,
    pub requires_elevation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemThemeInfo {
    pub color_scheme: String,
    pub gtk_theme: String,
    pub accent_hex: String,
    pub accent_dark_hex: String,
}
