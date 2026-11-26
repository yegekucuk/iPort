#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;

use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};

use crate::classifier::{classify_process, current_os, ClassificationInput};
use crate::enrichers::docker::DockerSnapshot;
use crate::enrichers::framework::FrameworkEnricher;
use crate::models::{
    AppCategory, DockerMeta, ManagedAction, PortEntry, PortSnapshot, Protocol, ServiceManager,
    SnapshotCounts, SourceType,
};

#[cfg(target_os = "linux")]
use linux::{collect_raw_sockets, parse_docker_proxy_cmdline};
#[cfg(target_os = "macos")]
use macos::collect_raw_sockets;
#[cfg(target_os = "windows")]
use windows::collect_raw_sockets;

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
struct RawSocketFallback {
    protocol: Protocol,
    port: u16,
    bind_address: String,
    state: String,
    pid: Option<u32>,
    uid: Option<u32>,
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn collect_raw_sockets() -> Vec<RawSocketFallback> {
    Vec::new()
}

#[derive(Debug, Clone)]
pub struct ProcSnapshot {
    pub pid: u32,
    pub ppid: Option<u32>,
    pub uid: Option<u32>,
    pub name: String,
    pub exe_path: Option<String>,
    pub command_line: String,
    pub cwd: Option<String>,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub uptime_secs: Option<u64>,
}

enum ResolvedProc<'a> {
    Matched(&'a ProcSnapshot),
    Synthetic {
        name: &'static str,
        command_line: &'static str,
    },
    Unresolved,
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
        self.scan_with_docker(&DockerSnapshot::default())
    }

    pub fn scan_with_docker(&mut self, docker: &DockerSnapshot) -> PortSnapshot {
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

        let procs: Vec<ProcSnapshot> = self
            .sys
            .processes()
            .iter()
            .map(|(pid, proc_info)| {
                let name = proc_info.name().to_string_lossy().into_owned();
                let exe_path = proc_info
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
                let uptime_secs = if start > 0 && now_secs >= start {
                    Some(now_secs - start)
                } else {
                    None
                };

                #[cfg(unix)]
                let uid = proc_info.user_id().map(|u| **u);
                #[cfg(not(unix))]
                let uid = None;

                ProcSnapshot {
                    pid: pid.as_u32(),
                    ppid: proc_info.parent().map(|p| p.as_u32()),
                    uid,
                    name,
                    exe_path,
                    command_line,
                    cwd,
                    cpu_percent: proc_info.cpu_usage(),
                    memory_bytes: proc_info.memory(),
                    uptime_secs,
                }
            })
            .collect();

        let proc_by_pid: HashMap<u32, &ProcSnapshot> =
            procs.iter().map(|p| (p.pid, p)).collect();
        let current_uid = current_process_uid();

        let raw_sockets = collect_raw_sockets();
        let os = current_os();
        let mut seen_ids = HashSet::new();
        let mut seen_socket_ports: HashSet<(Protocol, u16)> = HashSet::new();
        let mut seen_docker_keys: HashSet<(Protocol, String, u16)> = HashSet::new();
        let mut entries = Vec::with_capacity(raw_sockets.len());

        for sock in raw_sockets {
            seen_socket_ports.insert((sock.protocol, sock.port));
            let docker_meta = docker
                .lookup(sock.protocol, &sock.bind_address, sock.port)
                .cloned();
            if docker_meta.is_some() {
                seen_docker_keys.insert((sock.protocol, sock.bind_address.clone(), sock.port));
            }

            let resolved = match sock.pid.and_then(|p| proc_by_pid.get(&p).copied()) {
                Some(p) => ResolvedProc::Matched(p),
                None => resolve_unprivileged_process(
                    &procs,
                    sock.protocol,
                    &sock.bind_address,
                    sock.port,
                    sock.uid,
                    current_uid,
                    docker_meta.is_some(),
                ),
            };

            let (
                resolved_pid,
                ppid,
                process_name,
                exe_path,
                command_line,
                cwd,
                cpu_percent,
                memory_bytes,
                uptime_secs,
            ) = match resolved {
                ResolvedProc::Matched(proc_info) => (
                    Some(proc_info.pid),
                    proc_info.ppid,
                    proc_info.name.clone(),
                    proc_info.exe_path.clone(),
                    proc_info.command_line.clone(),
                    proc_info.cwd.clone(),
                    proc_info.cpu_percent,
                    proc_info.memory_bytes,
                    proc_info.uptime_secs,
                ),
                ResolvedProc::Synthetic { name, command_line } => (
                    None,
                    None,
                    name.to_string(),
                    None,
                    command_line.to_string(),
                    None,
                    0.0,
                    0,
                    None,
                ),
                ResolvedProc::Unresolved => {
                    let fallback_name = if sock.pid.is_some() {
                        "unknown".to_string()
                    } else {
                        "system/root".to_string()
                    };
                    (
                        sock.pid,
                        None,
                        fallback_name,
                        None,
                        String::new(),
                        None,
                        0.0,
                        0,
                        None,
                    )
                }
            };

            let pid_key = resolved_pid
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
                    uid: sock.uid,
                    is_docker: docker_meta.is_some(),
                    framework: enriched.framework.as_deref(),
                    runtime: enriched.runtime.as_deref(),
                },
                os,
            );

            let project_name = enriched.project_name.or_else(|| {
                docker_meta
                    .as_ref()
                    .and_then(|d| d.compose_project.clone())
            });

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
            if resolved_pid.is_some() {
                available_actions.push(ManagedAction::Stop);
                available_actions.push(ManagedAction::ForceKill);
            }

            entries.push(PortEntry {
                id,
                port: sock.port,
                protocol: sock.protocol,
                bind_address: sock.bind_address,
                state: sock.state,
                pid: resolved_pid,
                ppid,
                process_name,
                exe_path,
                command_line,
                cwd,
                project_name,
                cpu_percent,
                memory_bytes,
                uptime_secs,
                source_type: class_res.source_type,
                category: class_res.category,
                is_core_os: class_res.is_core_os,
                framework: enriched.framework,
                runtime: enriched.runtime,
                service_manager: class_res.service_manager,
                docker: docker_meta,
                available_actions,
            });
        }

        // Synthesize Docker published ports not directly observed in host raw sockets
        for (protocol, host_ip, host_port, meta) in &docker.published {
            if seen_docker_keys.contains(&(*protocol, host_ip.clone(), *host_port))
                || seen_socket_ports.contains(&(*protocol, *host_port))
            {
                continue;
            }

            let proto_str = match protocol {
                Protocol::Tcp => "tcp",
                Protocol::Udp => "udp",
            };
            let id = format!("{proto_str}:{host_port}:{host_ip}:docker-{}", meta.container_id);
            if !seen_ids.insert(id.clone()) {
                continue;
            }

            let mut available_actions = Vec::new();
            if *protocol == Protocol::Tcp {
                available_actions.push(ManagedAction::OpenBrowser);
            }
            available_actions.push(ManagedAction::CopyUrl);
            available_actions.push(ManagedAction::CopyPort);

            entries.push(build_synthetic_docker_entry(
                id,
                *port_ref(host_port),
                *protocol,
                host_ip.clone(),
                meta.clone(),
                available_actions,
            ));
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

fn port_ref(p: &u16) -> &u16 {
    p
}

fn build_synthetic_docker_entry(
    id: String,
    port: u16,
    protocol: Protocol,
    bind_address: String,
    meta: DockerMeta,
    available_actions: Vec<ManagedAction>,
) -> PortEntry {
    let project_name = meta.compose_project.clone();
    PortEntry {
        id,
        port,
        protocol,
        bind_address,
        state: "LISTEN".to_string(),
        pid: None,
        ppid: None,
        process_name: meta.container_name.clone(),
        exe_path: None,
        command_line: format!("docker container {} ({})", meta.container_name, meta.image),
        cwd: None,
        project_name,
        cpu_percent: 0.0,
        memory_bytes: 0,
        uptime_secs: None,
        source_type: SourceType::Docker,
        category: AppCategory::Docker,
        is_core_os: false,
        framework: None,
        runtime: None,
        service_manager: ServiceManager::Docker,
        docker: Some(meta),
        available_actions,
    }
}

fn current_process_uid() -> Option<u32> {
    #[cfg(unix)]
    {
        Some(unsafe { libc::getuid() as u32 })
    }
    #[cfg(not(unix))]
    {
        None
    }
}

fn resolve_unprivileged_process<'a>(
    procs: &'a [ProcSnapshot],
    protocol: Protocol,
    bind_address: &str,
    port: u16,
    socket_uid: Option<u32>,
    current_uid: Option<u32>,
    has_docker_meta: bool,
) -> ResolvedProc<'a> {
    // Tier A: Match root-owned docker-proxy directly from world-readable /proc/<pid>/cmdline
    #[cfg(target_os = "linux")]
    {
        let mut port_fallback: Option<&'a ProcSnapshot> = None;
        for proc_info in procs {
            if let Some((proxy_proto, proxy_ip, proxy_port)) =
                parse_docker_proxy_cmdline(&proc_info.command_line)
            {
                if proxy_proto == protocol && proxy_port == port {
                    if proxy_ip == bind_address {
                        return ResolvedProc::Matched(proc_info);
                    }
                    if port_fallback.is_none() {
                        port_fallback = Some(proc_info);
                    }
                }
            }
        }
        if let Some(matched) = port_fallback {
            return ResolvedProc::Matched(matched);
        }
    }

    // Avoid unused warning on non-Linux
    #[cfg(not(target_os = "linux"))]
    let _ = (protocol, bind_address);

    // Tier B: Dedicated non-root service UID (e.g., systemd-resolve=991, avahi=108, ollama=997)
    if let Some(uid) = socket_uid {
        if uid > 0 && Some(uid) != current_uid {
            let candidates: Vec<&ProcSnapshot> = procs
                .iter()
                .filter(|p| p.uid == Some(uid) && p.ppid != Some(2))
                .collect();
            if !candidates.is_empty() {
                let uid_pids: HashSet<u32> = candidates.iter().map(|p| p.pid).collect();
                let best = candidates
                    .into_iter()
                    .min_by_key(|p| {
                        let is_child_of_same_uid = p
                            .ppid
                            .map(|pp| uid_pids.contains(&pp))
                            .unwrap_or(false);
                        (is_child_of_same_uid, p.pid)
                    })
                    .unwrap();
                return ResolvedProc::Matched(best);
            }
        }
    }

    // Tier C: Well-known root daemon hints when socket UID is 0 and not a Docker proxy
    if socket_uid == Some(0) && !has_docker_meta {
        let hints = root_daemon_hints_for_port(port);
        if !hints.is_empty() {
            let matches: Vec<&ProcSnapshot> = procs
                .iter()
                .filter(|p| p.uid == Some(0) && matches_daemon_hint(p, hints))
                .collect();

            if !matches.is_empty() {
                let matched_pids: HashSet<u32> = matches.iter().map(|p| p.pid).collect();
                let best = matches
                    .into_iter()
                    .min_by_key(|p| {
                        let is_worker = p
                            .ppid
                            .map(|pp| matched_pids.contains(&pp))
                            .unwrap_or(false);
                        (is_worker, p.pid)
                    })
                    .unwrap();
                return ResolvedProc::Matched(best);
            }

            if port == 22 {
                return ResolvedProc::Synthetic {
                    name: "sshd (socket)",
                    command_line: "systemd socket activation (ssh.socket)",
                };
            }
        }
    }

    ResolvedProc::Unresolved
}

fn matches_daemon_hint(proc_info: &ProcSnapshot, hints: &[&str]) -> bool {
    let name_lower = proc_info.name.to_ascii_lowercase();
    let cmd_bin = proc_info
        .command_line
        .split_whitespace()
        .next()
        .and_then(|s| s.rsplit('/').next())
        .unwrap_or("")
        .to_ascii_lowercase();

    hints
        .iter()
        .any(|hint| name_lower == *hint || cmd_bin == *hint)
}

fn root_daemon_hints_for_port(port: u16) -> &'static [&'static str] {
    match port {
        22 => &["sshd"],
        53 => &[
            "systemd-resolved",
            "systemd-resolve",
            "dnsmasq",
            "named",
            "unbound",
        ],
        67 | 68 | 546 => &[
            "networkmanager",
            "systemd-networkd",
            "systemd-network",
            "dhclient",
            "dhcpcd",
        ],
        80 | 443 => &["apache2", "httpd", "nginx", "caddy", "traefik"],
        631 => &["cupsd", "cups-browsed"],
        3306 => &["mysqld", "mariadbd"],
        5353 => &["avahi-daemon"],
        5432 => &["postgres", "postmaster"],
        6379 => &["redis-server", "valkey-server"],
        11434 => &["ollama"],
        27017 => &["mongod"],
        _ => &[],
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
    #[cfg(target_os = "linux")]
    use std::net::TcpListener;

    fn fake_proc(pid: u32, ppid: u32, uid: u32, name: &str, cmd: &str) -> ProcSnapshot {
        ProcSnapshot {
            pid,
            ppid: Some(ppid),
            uid: Some(uid),
            name: name.to_string(),
            exe_path: None,
            command_line: cmd.to_string(),
            cwd: None,
            cpu_percent: 0.0,
            memory_bytes: 1024,
            uptime_secs: Some(60),
        }
    }

    #[test]
    fn resolves_unprivileged_service_uid_and_root_daemons() {
        let procs = vec![
            fake_proc(
                865,
                1,
                991,
                "systemd-resolve",
                "/usr/lib/systemd/systemd-resolved",
            ),
            fake_proc(969, 1, 108, "avahi-daemon", "avahi-daemon: running"),
            fake_proc(1037, 969, 108, "avahi-daemon", "avahi-daemon: chroot helper"),
            fake_proc(1475, 1, 0, "cupsd", "/usr/sbin/cupsd -l"),
            fake_proc(1524, 1, 0, "apache2", "/usr/sbin/apache2 -k start"),
            fake_proc(2312, 1, 997, "ollama", "/usr/local/bin/ollama serve"),
            fake_proc(
                3042,
                1,
                0,
                "docker-proxy",
                "/usr/bin/docker-proxy -proto tcp -host-ip 0.0.0.0 -host-port 8001 -container-ip 172.17.0.2 -container-port 11434 -use-listen-fd",
            ),
        ];

        // Docker proxy resolved by cmdline (Tier A is Linux-only)
        #[cfg(target_os = "linux")]
        match resolve_unprivileged_process(
            &procs,
            Protocol::Tcp,
            "0.0.0.0",
            8001,
            Some(0),
            Some(1000),
            true,
        ) {
            ResolvedProc::Matched(p) => assert_eq!(p.pid, 3042),
            _ => panic!("expected docker-proxy match"),
        }

        // Dedicated non-root service UID chooses parent process over child helper
        match resolve_unprivileged_process(
            &procs,
            Protocol::Udp,
            "0.0.0.0",
            5353,
            Some(108),
            Some(1000),
            false,
        ) {
            ResolvedProc::Matched(p) => assert_eq!(p.pid, 969),
            _ => panic!("expected avahi master match"),
        }

        match resolve_unprivileged_process(
            &procs,
            Protocol::Tcp,
            "127.0.0.1",
            11434,
            Some(997),
            Some(1000),
            false,
        ) {
            ResolvedProc::Matched(p) => assert_eq!(p.name, "ollama"),
            _ => panic!("expected ollama UID match"),
        }

        // Root daemon matched by port hint
        match resolve_unprivileged_process(
            &procs,
            Protocol::Tcp,
            "127.0.0.1",
            631,
            Some(0),
            Some(1000),
            false,
        ) {
            ResolvedProc::Matched(p) => assert_eq!(p.pid, 1475),
            _ => panic!("expected cupsd match"),
        }

        // Socket-activated sshd fallback when sshd process is absent
        match resolve_unprivileged_process(
            &procs,
            Protocol::Tcp,
            "0.0.0.0",
            22,
            Some(0),
            Some(1000),
            false,
        ) {
            ResolvedProc::Synthetic { name, .. } => assert_eq!(name, "sshd (socket)"),
            _ => panic!("expected sshd socket activation fallback"),
        }
    }

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

    #[cfg(target_os = "linux")]
    #[tokio::test]
    #[ignore = "inspects live host sockets and Docker daemon"]
    async fn live_host_scan_diagnostic() {
        let docker = crate::enrichers::docker::fetch_docker_snapshot().await;
        let mut scanner = PortScanner::new();
        let snap = scanner.scan_with_docker(&docker);
        println!("COUNTS: {:?}", snap.counts);
        for e in &snap.entries {
            println!(
                ":{:<5} {:?} {:<15} cat={:?} os={} pid={:?} proc={} docker={:?} cmd={}",
                e.port,
                e.protocol,
                e.bind_address,
                e.category,
                e.is_core_os,
                e.pid,
                e.process_name,
                e.docker.as_ref().map(|d| (&d.container_name, d.container_port)),
                e.command_line
            );
        }
        assert!(snap.counts.system > 0, "expected system ports > 0");
    }
}
