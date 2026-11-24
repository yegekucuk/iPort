use std::collections::HashMap;
use std::time::Duration;

use bollard::container::ListContainersOptions;
use bollard::models::{ContainerSummary, PortTypeEnum};
use bollard::Docker;

use crate::models::{DockerMeta, Protocol};

#[derive(Debug, Default, Clone)]
pub struct DockerSnapshot {
    pub exact: HashMap<(Protocol, String, u16), DockerMeta>,
    pub by_port: HashMap<(Protocol, u16), DockerMeta>,
    pub published: Vec<(Protocol, String, u16, DockerMeta)>,
}

impl DockerSnapshot {
    pub fn lookup(&self, protocol: Protocol, bind_address: &str, port: u16) -> Option<&DockerMeta> {
        self.exact
            .get(&(protocol, bind_address.to_string(), port))
            .or_else(|| self.by_port.get(&(protocol, port)))
    }
}

pub async fn fetch_docker_snapshot() -> DockerSnapshot {
    let Some(docker) = connect_docker() else {
        return DockerSnapshot::default();
    };

    let options = Some(ListContainersOptions::<String> {
        all: false,
        ..Default::default()
    });

    let Ok(Ok(containers)) =
        tokio::time::timeout(Duration::from_millis(800), docker.list_containers(options)).await
    else {
        return DockerSnapshot::default();
    };

    build_docker_snapshot(&containers)
}

fn connect_docker() -> Option<Docker> {
    #[cfg(unix)]
    {
        use std::path::{Path, PathBuf};

        if std::env::var_os("DOCKER_HOST").is_some() || Path::new("/var/run/docker.sock").exists() {
            if let Ok(client) = Docker::connect_with_local_defaults() {
                return Some(client);
            }
        }

        let mut candidates: Vec<PathBuf> = Vec::new();
        if let Some(runtime_dir) = std::env::var_os("XDG_RUNTIME_DIR") {
            candidates.push(PathBuf::from(runtime_dir).join("docker.sock"));
        }
        if let Some(home) = std::env::var_os("HOME") {
            let home_dir = PathBuf::from(home);
            candidates.push(home_dir.join(".docker/run/docker.sock"));
            candidates.push(home_dir.join(".orbstack/run/docker.sock"));
            candidates.push(home_dir.join(".colima/default/docker.sock"));
        }

        for sock_path in candidates {
            if sock_path.exists() {
                if let Some(path_str) = sock_path.to_str() {
                    if let Ok(client) =
                        Docker::connect_with_socket(path_str, 3, bollard::API_DEFAULT_VERSION)
                    {
                        return Some(client);
                    }
                }
            }
        }

        None
    }

    #[cfg(not(unix))]
    {
        Docker::connect_with_local_defaults().ok()
    }
}

pub fn build_docker_snapshot(containers: &[ContainerSummary]) -> DockerSnapshot {
    let mut snapshot = DockerSnapshot::default();

    for container in containers {
        let Some(ports) = container.ports.as_ref() else {
            continue;
        };

        let container_id = container
            .id
            .as_deref()
            .map(|id| id.chars().take(12).collect::<String>())
            .unwrap_or_default();

        let container_name = container
            .names
            .as_ref()
            .and_then(|names| names.first())
            .map(|name| name.trim_start_matches('/').to_string())
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| {
                if container_id.is_empty() {
                    "docker-container".to_string()
                } else {
                    container_id.clone()
                }
            });

        let image = container.image.clone().unwrap_or_default();
        let status = container.status.clone();
        let labels = container.labels.as_ref();
        let compose_project = labels
            .and_then(|l| l.get("com.docker.compose.project"))
            .cloned();
        let compose_service = labels
            .and_then(|l| l.get("com.docker.compose.service"))
            .cloned();

        for port in ports {
            let Some(host_port) = port.public_port else {
                continue;
            };
            if host_port == 0 {
                continue;
            }

            let protocol = match port.typ {
                Some(PortTypeEnum::UDP) => Protocol::Udp,
                _ => Protocol::Tcp,
            };

            let host_ip = port
                .ip
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .unwrap_or("0.0.0.0")
                .to_string();

            let meta = DockerMeta {
                container_id: container_id.clone(),
                container_name: container_name.clone(),
                image: image.clone(),
                status: status.clone(),
                container_port: port.private_port,
                compose_project: compose_project.clone(),
                compose_service: compose_service.clone(),
            };

            snapshot
                .exact
                .entry((protocol, host_ip.clone(), host_port))
                .or_insert_with(|| meta.clone());
            snapshot
                .by_port
                .entry((protocol, host_port))
                .or_insert_with(|| meta.clone());
            snapshot
                .published
                .push((protocol, host_ip, host_port, meta));
        }
    }

    snapshot
}

#[cfg(test)]
mod tests {
    use super::*;
    use bollard::models::Port;

    #[test]
    fn builds_and_looks_up_docker_port_mappings() {
        let mut labels = HashMap::new();
        labels.insert("com.docker.compose.project".to_string(), "hermes".to_string());
        labels.insert("com.docker.compose.service".to_string(), "searxng".to_string());

        let containers = vec![
            ContainerSummary {
                id: Some("abcdef1234567890".to_string()),
                names: Some(vec!["/hermes-searxng".to_string()]),
                image: Some("searxng/searxng".to_string()),
                status: Some("Up 3 hours".to_string()),
                labels: Some(labels),
                ports: Some(vec![
                    Port {
                        ip: Some("0.0.0.0".to_string()),
                        private_port: 8080,
                        public_port: Some(8888),
                        typ: Some(PortTypeEnum::TCP),
                    },
                    Port {
                        ip: Some("::".to_string()),
                        private_port: 8080,
                        public_port: Some(8888),
                        typ: Some(PortTypeEnum::TCP),
                    },
                ]),
                ..Default::default()
            },
            ContainerSummary {
                id: Some("9988776655443322".to_string()),
                names: Some(vec!["/smollm".to_string()]),
                image: Some("yegekucuk2/smollm:360m".to_string()),
                status: Some("Up 3 hours".to_string()),
                ports: Some(vec![Port {
                    ip: Some("0.0.0.0".to_string()),
                    private_port: 11434,
                    public_port: Some(8001),
                    typ: Some(PortTypeEnum::TCP),
                }]),
                ..Default::default()
            },
        ];

        let snap = build_docker_snapshot(&containers);

        let searx = snap.lookup(Protocol::Tcp, "0.0.0.0", 8888).expect("exact match");
        assert_eq!(searx.container_id, "abcdef123456");
        assert_eq!(searx.container_name, "hermes-searxng");
        assert_eq!(searx.image, "searxng/searxng");
        assert_eq!(searx.container_port, 8080);
        assert_eq!(searx.compose_project.as_deref(), Some("hermes"));
        assert_eq!(searx.compose_service.as_deref(), Some("searxng"));

        // Falls back cleanly by (protocol, host_port) when IP differs
        let smollm = snap.lookup(Protocol::Tcp, "::", 8001).expect("port fallback");
        assert_eq!(smollm.container_name, "smollm");
        assert_eq!(smollm.container_port, 11434);
    }
}
