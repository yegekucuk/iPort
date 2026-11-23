use crate::models::{AppCategory, ServiceManager, SourceType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetOs {
    Linux,
    MacOS,
    Windows,
}

pub fn current_os() -> TargetOs {
    #[cfg(target_os = "macos")]
    {
        TargetOs::MacOS
    }
    #[cfg(target_os = "windows")]
    {
        TargetOs::Windows
    }
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    {
        TargetOs::Linux
    }
}

pub struct ClassificationInput<'a> {
    pub process_name: &'a str,
    pub exe_path: Option<&'a str>,
    pub command_line: &'a str,
    pub port: u16,
    pub framework: Option<&'a str>,
    pub runtime: Option<&'a str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClassificationResult {
    pub source_type: SourceType,
    pub category: AppCategory,
    pub is_core_os: bool,
    pub service_manager: ServiceManager,
}

const LINUX_CORE_DAEMONS: &[&str] = &[
    "systemd-resolved",
    "systemd-timesyncd",
    "systemd-networkd",
    "systemd-journald",
    "systemd-udevd",
    "avahi-daemon",
    "cupsd",
    "cups-browsed",
    "dhclient",
    "dhcpcd",
    "networkmanager",
    "wpa_supplicant",
    "rpcbind",
    "rpc.statd",
    "rpc.mountd",
    "chronyd",
    "ntpd",
    "bluetoothd",
    "sshd",
    "rsyslogd",
    "snapd",
    "packagekitd",
    "colord",
    "udisksd",
    "polkitd",
    "dbus-daemon",
    "pipewire",
    "pulseaudio",
];

const MACOS_CORE_DAEMONS: &[&str] = &[
    "mdnsresponder",
    "controlcenter",
    "rapportd",
    "airplayxpchelper",
    "sharingd",
    "apsd",
    "configd",
    "usereventagent",
    "launchd",
    "symptomsd",
    "netbiosd",
    "krb5kdc",
    "cupsd",
    "sshd",
    "identityservicesd",
    "bluetoothd",
    "coreaudiod",
    "remoted",
];

const WINDOWS_CORE_DAEMONS: &[&str] = &[
    "system",
    "idle",
    "svchost.exe",
    "svchost",
    "lsass.exe",
    "lsass",
    "services.exe",
    "services",
    "spoolsv.exe",
    "spoolsv",
    "wininit.exe",
    "wininit",
    "csrss.exe",
    "smss.exe",
    "dashost.exe",
    "searchindexer.exe",
    "dllhost.exe",
];

const DEV_BINARIES: &[&str] = &[
    "node",
    "nodejs",
    "bun",
    "deno",
    "python",
    "python3",
    "uvicorn",
    "gunicorn",
    "gunicorn3",
    "ruby",
    "rails",
    "puma",
    "go",
    "air",
    "cargo",
    "java",
    "dotnet",
    "php",
    "php-fpm",
    "postgres",
    "postmaster",
    "mysqld",
    "mariadbd",
    "mongod",
    "mongos",
    "redis-server",
    "valkey-server",
    "elasticsearch",
    "beam.smp",
    "rabbitmq-server",
    "minio",
    "vite",
    "esbuild",
    "next-server",
    "nodemon",
    "tsx",
    "caddy",
    "nginx",
    "traefik",
    "httpd",
    "apache2",
    "lighttpd",
];

pub fn classify_process(input: &ClassificationInput<'_>, os: TargetOs) -> ClassificationResult {
    let name_lower = input.process_name.trim().to_ascii_lowercase();
    let exe_lower = input
        .exe_path
        .map(|s| s.trim().to_ascii_lowercase())
        .unwrap_or_default();
    let cmd_lower = input.command_line.trim().to_ascii_lowercase();

    if is_tunnel(&name_lower, &cmd_lower) {
        return ClassificationResult {
            source_type: SourceType::Tunnel,
            category: AppCategory::Tunnel,
            is_core_os: false,
            service_manager: ServiceManager::None,
        };
    }

    if is_docker_proxy(&name_lower, &exe_lower, &cmd_lower) {
        return ClassificationResult {
            source_type: SourceType::Docker,
            category: AppCategory::Docker,
            is_core_os: false,
            service_manager: ServiceManager::Docker,
        };
    }

    if is_pm2_managed(&name_lower, &exe_lower, &cmd_lower) {
        return ClassificationResult {
            source_type: SourceType::Service,
            category: AppCategory::Service,
            is_core_os: false,
            service_manager: ServiceManager::Pm2,
        };
    }

    if is_core_os_process(&name_lower, &exe_lower, &cmd_lower, input.port, os)
        && input.framework.is_none()
        && input.runtime.is_none()
    {
        let service_manager = if os == TargetOs::Linux && name_lower.starts_with("systemd") {
            ServiceManager::Systemd
        } else if os == TargetOs::MacOS && exe_lower.starts_with("/system/") {
            ServiceManager::Launchd
        } else if os == TargetOs::Windows && name_lower.starts_with("svchost") {
            ServiceManager::WindowsService
        } else {
            ServiceManager::None
        };

        return ClassificationResult {
            source_type: SourceType::System,
            category: AppCategory::System,
            is_core_os: true,
            service_manager,
        };
    }

    if input.framework.is_some()
        || input.runtime.is_some()
        || DEV_BINARIES.iter().any(|bin| name_lower == *bin)
        || (name_lower.starts_with("python3.") && !name_lower.contains("apport"))
    {
        return ClassificationResult {
            source_type: SourceType::Local,
            category: AppCategory::Dev,
            is_core_os: false,
            service_manager: ServiceManager::None,
        };
    }

    ClassificationResult {
        source_type: SourceType::ThirdParty,
        category: AppCategory::ThirdParty,
        is_core_os: false,
        service_manager: ServiceManager::None,
    }
}

fn is_tunnel(name_lower: &str, cmd_lower: &str) -> bool {
    if matches!(name_lower, "ngrok" | "cloudflared" | "autossh" | "chisel" | "bore") {
        return true;
    }
    if matches!(name_lower, "kubectl" | "kubectl.exe" | "oc") && cmd_lower.contains("port-forward") {
        return true;
    }
    if matches!(name_lower, "ssh" | "ssh.exe")
        && (has_cli_flag(cmd_lower, "-l")
            || has_cli_flag(cmd_lower, "-r")
            || has_cli_flag(cmd_lower, "-d"))
    {
        return true;
    }
    false
}

fn has_cli_flag(cmd_lower: &str, flag: &str) -> bool {
    cmd_lower
        .split_whitespace()
        .any(|tok| tok == flag || tok.starts_with(flag))
}

fn is_docker_proxy(name_lower: &str, exe_lower: &str, cmd_lower: &str) -> bool {
    if matches!(
        name_lower,
        "docker-proxy"
            | "com.docker.backend"
            | "com.docker.vpnkit"
            | "vpnkit"
            | "vpnkit-bridge"
            | "orbstack"
            | "orbstack-helper"
            | "colima"
            | "limactl"
            | "rootlesskit"
    ) {
        return true;
    }
    if exe_lower.contains("docker") && (name_lower.contains("proxy") || name_lower.contains("vpnkit"))
    {
        return true;
    }
    if name_lower.starts_with("qemu-system")
        && (cmd_lower.contains("colima")
            || cmd_lower.contains("orbstack")
            || cmd_lower.contains("rancher")
            || cmd_lower.contains("lima"))
    {
        return true;
    }
    false
}

fn is_pm2_managed(name_lower: &str, exe_lower: &str, cmd_lower: &str) -> bool {
    name_lower == "pm2"
        || name_lower.starts_with("pm2 ")
        || cmd_lower.contains("/pm2/")
        || cmd_lower.contains("\\pm2\\")
        || exe_lower.contains("/pm2/")
}

fn is_core_os_process(
    name_lower: &str,
    exe_lower: &str,
    cmd_lower: &str,
    port: u16,
    os: TargetOs,
) -> bool {
    // Explicit user/third-party installation prefixes are never treated as core OS daemons.
    if exe_lower.starts_with("/home/")
        || exe_lower.starts_with("/users/")
        || exe_lower.starts_with("/opt/")
        || exe_lower.starts_with("/usr/local/")
        || exe_lower.starts_with("/applications/")
        || exe_lower.starts_with("/snap/")
        || exe_lower.contains("/.local/")
        || exe_lower.contains("program files")
        || exe_lower.contains("appdata")
    {
        return false;
    }

    match os {
        TargetOs::Linux => {
            if LINUX_CORE_DAEMONS.iter().any(|d| name_lower == *d) {
                return true;
            }
            if name_lower == "dnsmasq"
                && (cmd_lower.contains("networkmanager")
                    || cmd_lower.contains("libvirt")
                    || exe_lower.starts_with("/usr/sbin/"))
            {
                return true;
            }
            if port == 22 && name_lower.contains("sshd") {
                return true;
            }
            exe_lower.starts_with("/lib/systemd/")
                || exe_lower.starts_with("/usr/lib/systemd/")
                || exe_lower.starts_with("/usr/libexec/")
        }
        TargetOs::MacOS => {
            if MACOS_CORE_DAEMONS.iter().any(|d| name_lower == *d) {
                return true;
            }
            if port == 22 && name_lower.contains("sshd") {
                return true;
            }
            exe_lower.starts_with("/system/") || exe_lower.starts_with("/usr/libexec/")
        }
        TargetOs::Windows => {
            if WINDOWS_CORE_DAEMONS.iter().any(|d| name_lower == *d) {
                return true;
            }
            let normalized = exe_lower.replace('/', "\\");
            normalized.contains("\\windows\\system32\\")
                || normalized.contains("\\windows\\syswow64\\")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_linux_core_os_daemons_as_system() {
        let res = classify_process(
            &ClassificationInput {
                process_name: "systemd-resolved",
                exe_path: Some("/usr/lib/systemd/systemd-resolved"),
                command_line: "/usr/lib/systemd/systemd-resolved",
                port: 53,
                framework: None,
                runtime: None,
            },
            TargetOs::Linux,
        );
        assert!(res.is_core_os);
        assert_eq!(res.category, AppCategory::System);
        assert_eq!(res.source_type, SourceType::System);
        assert_eq!(res.service_manager, ServiceManager::Systemd);
    }

    #[test]
    fn surfaces_third_party_apps_by_default_on_linux() {
        let res = classify_process(
            &ClassificationInput {
                process_name: "ollama",
                exe_path: Some("/usr/local/bin/ollama"),
                command_line: "/usr/local/bin/ollama serve",
                port: 11434,
                framework: None,
                runtime: None,
            },
            TargetOs::Linux,
        );
        assert!(!res.is_core_os);
        assert_eq!(res.category, AppCategory::ThirdParty);
        assert_eq!(res.source_type, SourceType::ThirdParty);
    }

    #[test]
    fn classifies_macos_core_vs_homebrew_third_party() {
        let os_res = classify_process(
            &ClassificationInput {
                process_name: "mDNSResponder",
                exe_path: Some("/usr/sbin/mDNSResponder"),
                command_line: "/usr/sbin/mDNSResponder",
                port: 5353,
                framework: None,
                runtime: None,
            },
            TargetOs::MacOS,
        );
        assert!(os_res.is_core_os);
        assert_eq!(os_res.category, AppCategory::System);

        let third_party = classify_process(
            &ClassificationInput {
                process_name: "Spotify",
                exe_path: Some("/Applications/Spotify.app/Contents/MacOS/Spotify"),
                command_line: "/Applications/Spotify.app/Contents/MacOS/Spotify",
                port: 57621,
                framework: None,
                runtime: None,
            },
            TargetOs::MacOS,
        );
        assert!(!third_party.is_core_os);
        assert_eq!(third_party.category, AppCategory::ThirdParty);
    }

    #[test]
    fn classifies_windows_system_vs_user_app() {
        let win_os = classify_process(
            &ClassificationInput {
                process_name: "svchost.exe",
                exe_path: Some(r"C:\Windows\System32\svchost.exe"),
                command_line: r"C:\Windows\System32\svchost.exe -k netsvcs",
                port: 135,
                framework: None,
                runtime: None,
            },
            TargetOs::Windows,
        );
        assert!(win_os.is_core_os);
        assert_eq!(win_os.category, AppCategory::System);
        assert_eq!(win_os.service_manager, ServiceManager::WindowsService);

        let win_dev = classify_process(
            &ClassificationInput {
                process_name: "node.exe",
                exe_path: Some(r"C:\Program Files\nodejs\node.exe"),
                command_line: "node server.js",
                port: 3000,
                framework: Some("Next.js"),
                runtime: Some("Node.js"),
            },
            TargetOs::Windows,
        );
        assert!(!win_dev.is_core_os);
        assert_eq!(win_dev.category, AppCategory::Dev);
    }

    #[test]
    fn classifies_tunnels_docker_and_pm2() {
        let ssh_tunnel = classify_process(
            &ClassificationInput {
                process_name: "ssh",
                exe_path: Some("/usr/bin/ssh"),
                command_line: "ssh -N -L 5433:localhost:5432 prod-bastion",
                port: 5433,
                framework: None,
                runtime: None,
            },
            TargetOs::Linux,
        );
        assert_eq!(ssh_tunnel.category, AppCategory::Tunnel);
        assert!(!ssh_tunnel.is_core_os);

        let kubectl = classify_process(
            &ClassificationInput {
                process_name: "kubectl",
                exe_path: Some("/usr/local/bin/kubectl"),
                command_line: "kubectl port-forward svc/api 8080:80 -n staging",
                port: 8080,
                framework: None,
                runtime: None,
            },
            TargetOs::Linux,
        );
        assert_eq!(kubectl.category, AppCategory::Tunnel);

        let docker = classify_process(
            &ClassificationInput {
                process_name: "docker-proxy",
                exe_path: Some("/usr/bin/docker-proxy"),
                command_line: "/usr/bin/docker-proxy -proto tcp -host-port 5432",
                port: 5432,
                framework: None,
                runtime: None,
            },
            TargetOs::Linux,
        );
        assert_eq!(docker.category, AppCategory::Docker);
        assert_eq!(docker.service_manager, ServiceManager::Docker);

        let pm2 = classify_process(
            &ClassificationInput {
                process_name: "node",
                exe_path: Some("/usr/bin/node"),
                command_line: "node /home/user/.npm-global/lib/node_modules/pm2/bin/pm2",
                port: 4000,
                framework: Some("Express"),
                runtime: Some("Node.js"),
            },
            TargetOs::Linux,
        );
        assert_eq!(pm2.category, AppCategory::Service);
        assert_eq!(pm2.service_manager, ServiceManager::Pm2);
    }
}
