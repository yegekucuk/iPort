#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;

use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

use crate::classifier::{classify_process, current_os, ClassificationInput};
use crate::enrichers::framework::FrameworkEnricher;
use crate::models::{
    AppCategory, ManagedAction, PortEntry, PortSnapshot, Protocol, SnapshotCounts,
};

#[cfg(target_os = "linux")]
use linux::collect_raw_sockets;
#[cfg(target_os = "macos")]
use macos::collect_raw_sockets;
#[cfg(target_os = "windows")]
use windows::collect_raw_sockets;

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn collect_raw_sockets() -> Vec<RawSocketFallback> {
    Vec::new()
}

pub struct PortScanner {
    sys: System,
    enricher: FrameworkEnricher,
}

impl Default for PortScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl PortScanner {
    pub fn new() -> Self {
        Self {
            sys: System::new(),
            enricher: FrameworkEnricher::new(),
        }
    }

    pub fn scan(&mut self) -> PortSnapshot {
        self.sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::everything(),
        );

        let now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let raw_sockets = collect_raw_sockets();
        let os = current_os();
        let mut seen_ids = HashSet::new();
        let mut entries = Vec::with_capacity(raw_sockets.len());

        for sock in raw_sockets {
            let pid_key = sock
                .pid
                .map(|p| p.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            let proto_str = match sock.protocol {
                Protocol::Tcp => "tcp",
                Protocol::Udp => "udp",
            };
            let id = format!("{proto_str}:{}:{}:{pid_key}", sock.port, sock.bind_address);
            if !seen_ids.insert(id.clone()) {
                continue;
            }

            let proc_opt = sock.pid.and_then(|p| self.sys.process(Pid::from_u32(p)));

            let (ppid, process_name, exe_path, command_line, cwd, cpu_percent, memory_bytes, uptime_secs) =
                if let Some(proc_info) = proc_opt {
                    let name = proc_info.name().to_string_lossy().into_owned();
                    let exe = proc_info
                        .exe()
                        .map(|p| p.to_string_lossy().into_owned())
                        .filter(|s| !s.is_empty());
                    let cmd_joined = proc_info
                        .cmd()
                        .iter()
                        .map(|s| s.to_string_lossy().into_owned())
                        .collect::<Vec<_>>()
                        .join(" ");
                    let command_line = if cmd_joined.trim().is_empty() {
                        name.clone()
                    } else {
                        cmd_joined
                    };
                    let cwd = proc_info
                        .cwd()
                        .map(|p| p.to_string_lossy().into_owned())
                        .filter(|s| !s.is_empty());
                    let start = proc_info.start_time();
                    let uptime = if start > 0 && now_secs >= start {
                        Some(now_secs - start)
                    } else {
                        None
                    };

                    (
                        proc_info.parent().map(|p| p.as_u32()),
                        name,
                        exe,
                        command_line,
                        cwd,
                        proc_info.cpu_usage(),
                        proc_info.memory(),
                        uptime,
                    )
                } else {
                    let fallback_name = if sock.pid.is_some() {
                        "unknown".to_string()
                    } else {
                        "system/root".to_string()
                    };
                    (None, fallback_name, None, String::new(), None, 0.0, 0, None)
                };

            let enriched = self.enricher.enrich(
                &process_name,
                exe_path.as_deref(),
                &command_line,
                cwd.as_deref(),
            );

            let class_res = classify_process(
                &ClassificationInput {
                    process_name: &process_name,
                    exe_path: exe_path.as_deref(),
                    command_line: &command_line,
                    port: sock.port,
                    framework: enriched.framework.as_deref(),
                    runtime: enriched.runtime.as_deref(),
                },
                os,
            );

            let mut available_actions = Vec::new();
            if sock.protocol == Protocol::Tcp {
                available_actions.push(ManagedAction::OpenBrowser);
            }
            available_actions.push(ManagedAction::CopyUrl);
            available_actions.push(ManagedAction::CopyPort);
            if cwd.is_some() {
                available_actions.push(ManagedAction::RevealFolder);
                available_actions.push(ManagedAction::OpenIde);
            }
            if sock.pid.is_some() {
                available_actions.push(ManagedAction::Stop);
                available_actions.push(ManagedAction::ForceKill);
            }

            entries.push(PortEntry {
                id,
                port: sock.port,
                protocol: sock.protocol,
                bind_address: sock.bind_address,
                state: sock.state,
                pid: sock.pid,
                ppid,
                process_name,
                exe_path,
                command_line,
                cwd,
                project_name: enriched.project_name,
                cpu_percent,
                memory_bytes,
                uptime_secs,
                source_type: class_res.source_type,
                category: class_res.category,
                is_core_os: class_res.is_core_os,
                framework: enriched.framework,
                runtime: enriched.runtime,
                service_manager: class_res.service_manager,
                available_actions,
            });
        }

        entries.sort_by(|a, b| {
            a.port
                .cmp(&b.port)
                .then(a.protocol.cmp(&b.protocol))
                .then(a.bind_address.cmp(&b.bind_address))
        });

        let counts = compute_counts(&entries);

        PortSnapshot {
            timestamp_ms,
            entries,
            counts,
        }
    }
}

fn compute_counts(entries: &[PortEntry]) -> SnapshotCounts {
    let mut counts = SnapshotCounts::default();
    for entry in entries {
        if entry.is_core_os {
            counts.system += 1;
            continue;
        }
        counts.total_visible += 1;
        match entry.category {
            AppCategory::Dev => counts.dev += 1,
            AppCategory::Docker => counts.docker += 1,
            AppCategory::Service => counts.services += 1,
            AppCategory::Tunnel => counts.tunnels += 1,
            AppCategory::ThirdParty => counts.third_party += 1,
            AppCategory::System => counts.system += 1,
        }
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    #[cfg(target_os = "linux")]
    #[test]
    fn scans_real_listening_tcp_socket() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
        let bound_port = listener.local_addr().expect("read local addr").port();
        let current_pid = std::process::id();

        let mut scanner = PortScanner::new();
        let snapshot = scanner.scan();

        let found = snapshot
            .entries
            .iter()
            .find(|e| e.port == bound_port && e.protocol == Protocol::Tcp);

        assert!(
            found.is_some(),
            "expected listening port {bound_port} to appear in scan snapshot"
        );
        let entry = found.unwrap();
        assert_eq!(entry.bind_address, "127.0.0.1");
        assert_eq!(entry.pid, Some(current_pid));
        assert!(!entry.is_core_os);
    }
}
