use std::collections::HashMap;
use std::fs;
use std::net::{Ipv4Addr, Ipv6Addr};

use crate::models::Protocol;

#[derive(Debug, Clone)]
pub struct RawSocket {
    pub protocol: Protocol,
    pub port: u16,
    pub bind_address: String,
    pub state: String,
    pub pid: Option<u32>,
    pub uid: Option<u32>,
}

pub fn collect_raw_sockets() -> Vec<RawSocket> {
    let inode_map = build_inode_pid_map();
    let mut sockets = Vec::new();

    if let Ok(content) = fs::read_to_string("/proc/net/tcp") {
        sockets.extend(parse_proc_net_table(&content, Protocol::Tcp, false, &inode_map));
    }
    if let Ok(content) = fs::read_to_string("/proc/net/tcp6") {
        sockets.extend(parse_proc_net_table(&content, Protocol::Tcp, true, &inode_map));
    }
    if let Ok(content) = fs::read_to_string("/proc/net/udp") {
        sockets.extend(parse_proc_net_table(&content, Protocol::Udp, false, &inode_map));
    }
    if let Ok(content) = fs::read_to_string("/proc/net/udp6") {
        sockets.extend(parse_proc_net_table(&content, Protocol::Udp, true, &inode_map));
    }

    sockets
}

fn build_inode_pid_map() -> HashMap<u64, u32> {
    let mut map = HashMap::new();
    let Ok(proc_entries) = fs::read_dir("/proc") else {
        return map;
    };

    for entry in proc_entries.flatten() {
        let file_name = entry.file_name();
        let Some(pid_str) = file_name.to_str() else {
            continue;
        };
        let Ok(pid) = pid_str.parse::<u32>() else {
            continue;
        };

        let fd_dir = entry.path().join("fd");
        let Ok(fds) = fs::read_dir(fd_dir) else {
            continue;
        };

        for fd in fds.flatten() {
            let Ok(target) = fs::read_link(fd.path()) else {
                continue;
            };
            let Some(target_str) = target.to_str() else {
                continue;
            };
            if let Some(inode_str) = target_str
                .strip_prefix("socket:[")
                .and_then(|s| s.strip_suffix(']'))
            {
                if let Ok(inode) = inode_str.parse::<u64>() {
                    map.entry(inode).or_insert(pid);
                }
            }
        }
    }

    map
}

pub fn parse_proc_net_table(
    content: &str,
    protocol: Protocol,
    is_v6: bool,
    inode_map: &HashMap<u64, u32>,
) -> Vec<RawSocket> {
    let mut results = Vec::new();

    for line in content.lines().skip(1) {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 10 {
            continue;
        }

        let state_hex = fields[3];
        let Some((local_ip_hex, local_port_hex)) = fields[1].split_once(':') else {
            continue;
        };
        let Some((_, remote_port_hex)) = fields[2].split_once(':') else {
            continue;
        };

        let Ok(local_port) = u16::from_str_radix(local_port_hex, 16) else {
            continue;
        };
        let Ok(remote_port) = u16::from_str_radix(remote_port_hex, 16) else {
            continue;
        };

        let (keep, state_label) = match protocol {
            Protocol::Tcp => {
                if state_hex == "0A" {
                    (true, "LISTEN".to_string())
                } else {
                    (false, String::new())
                }
            }
            Protocol::Udp => {
                // Keep listening/bound UDP sockets (state 07, non-zero local port, zero remote port)
                if state_hex == "07" && local_port > 0 && remote_port == 0 {
                    (true, "UNCONN".to_string())
                } else {
                    (false, String::new())
                }
            }
        };

        if !keep || local_port == 0 {
            continue;
        }

        let bind_address = if is_v6 {
            decode_ipv6_hex(local_ip_hex)
        } else {
            decode_ipv4_hex(local_ip_hex)
        };
        let Some(bind_address) = bind_address else {
            continue;
        };

        let uid = fields[7].parse::<u32>().ok();
        let inode = fields[9].parse::<u64>().ok();
        let pid = inode.and_then(|ino| inode_map.get(&ino).copied());

        results.push(RawSocket {
            protocol,
            port: local_port,
            bind_address,
            state: state_label,
            pid,
            uid,
        });
    }

    results
}

pub fn parse_docker_proxy_cmdline(cmd: &str) -> Option<(Protocol, String, u16)> {
    let tokens: Vec<&str> = cmd.split_whitespace().collect();
    if tokens.is_empty() || !tokens[0].ends_with("docker-proxy") {
        return None;
    }

    let mut protocol = Protocol::Tcp;
    let mut host_ip = String::from("0.0.0.0");
    let mut host_port: Option<u16> = None;

    let mut i = 1;
    while i + 1 < tokens.len() {
        match tokens[i] {
            "-proto" => {
                protocol = match tokens[i + 1].to_ascii_lowercase().as_str() {
                    "udp" => Protocol::Udp,
                    _ => Protocol::Tcp,
                };
                i += 2;
            }
            "-host-ip" => {
                host_ip = tokens[i + 1].to_string();
                i += 2;
            }
            "-host-port" => {
                host_port = tokens[i + 1].parse::<u16>().ok();
                i += 2;
            }
            _ => {
                i += 1;
            }
        }
    }

    host_port.map(|port| (protocol, host_ip, port))
}

pub fn decode_ipv4_hex(hex: &str) -> Option<String> {
    if hex.len() != 8 {
        return None;
    }
    let raw = u32::from_str_radix(hex, 16).ok()?;
    let addr = Ipv4Addr::from(raw.to_ne_bytes());
    Some(addr.to_string())
}

pub fn decode_ipv6_hex(hex: &str) -> Option<String> {
    if hex.len() != 32 {
        return None;
    }
    let mut octets = [0u8; 16];
    for i in 0..4 {
        let start = i * 8;
        let word = u32::from_str_radix(&hex[start..start + 8], 16).ok()?;
        let bytes = word.to_ne_bytes();
        octets[i * 4..i * 4 + 4].copy_from_slice(&bytes);
    }
    let addr = Ipv6Addr::from(octets);
    if let Some(v4) = addr.to_ipv4_mapped() {
        return Some(v4.to_string());
    }
    Some(addr.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_ipv4_hex_little_endian_proc_format() {
        assert_eq!(decode_ipv4_hex("0100007F").as_deref(), Some("127.0.0.1"));
        assert_eq!(decode_ipv4_hex("00000000").as_deref(), Some("0.0.0.0"));
        assert_eq!(decode_ipv4_hex("3500007F").as_deref(), Some("127.0.0.53"));
        assert_eq!(decode_ipv4_hex("INVALID"), None);
    }

    #[test]
    fn decodes_ipv6_hex_including_loopback_and_v4_mapped() {
        assert_eq!(
            decode_ipv6_hex("00000000000000000000000000000000").as_deref(),
            Some("::")
        );
        assert_eq!(
            decode_ipv6_hex("00000000000000000000000001000000").as_deref(),
            Some("::1")
        );
        assert_eq!(
            decode_ipv6_hex("0000000000000000FFFF00000100007F").as_deref(),
            Some("127.0.0.1")
        );
    }

    #[test]
    fn parses_proc_net_tcp_and_udp_tables_with_uid() {
        let tcp_sample = "\
  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode
   0: 0100007F:0BB8 00000000:0000 0A 00000000:00000000 00:00000000 00000000  1000        0 55100 1 0000000000000000 100 0 0 10 0
   1: 0100007F:0BB8 0100007F:D431 01 00000000:00000000 00:00000000 00000000  1000        0 55101 1 0000000000000000 100 0 0 10 0
   2: 00000000:1F40 00000000:0000 0A 00000000:00000000 00:00000000 00000000     0        0 99001 1 0000000000000000 100 0 0 10 0
";
        let udp_sample = "\
  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode ref pointer drops
   0: 3500007F:0035 00000000:0000 07 00000000:00000000 00:00000000 00000000   991        0 21450 2 0000000000000000 0
   1: 0100007F:9999 0100007F:0035 01 00000000:00000000 00:00000000 00000000  1000        0 21451 2 0000000000000000 0
";

        let mut inodes = HashMap::new();
        inodes.insert(55100, 4120);
        inodes.insert(21450, 865);

        let tcp = parse_proc_net_table(tcp_sample, Protocol::Tcp, false, &inodes);
        assert_eq!(tcp.len(), 2);
        assert_eq!(tcp[0].port, 3000);
        assert_eq!(tcp[0].bind_address, "127.0.0.1");
        assert_eq!(tcp[0].pid, Some(4120));
        assert_eq!(tcp[0].uid, Some(1000));
        assert_eq!(tcp[1].port, 8000);
        assert_eq!(tcp[1].bind_address, "0.0.0.0");
        assert_eq!(tcp[1].pid, None);
        assert_eq!(tcp[1].uid, Some(0));

        let udp = parse_proc_net_table(udp_sample, Protocol::Udp, false, &inodes);
        assert_eq!(udp.len(), 1);
        assert_eq!(udp[0].port, 53);
        assert_eq!(udp[0].bind_address, "127.0.0.53");
        assert_eq!(udp[0].pid, Some(865));
        assert_eq!(udp[0].uid, Some(991));
    }

    #[test]
    fn parses_docker_proxy_command_lines() {
        let v4 = parse_docker_proxy_cmdline(
            "/usr/bin/docker-proxy -proto tcp -host-ip 0.0.0.0 -host-port 8001 -container-ip 172.17.0.2 -container-port 11434 -use-listen-fd",
        );
        assert_eq!(v4, Some((Protocol::Tcp, "0.0.0.0".to_string(), 8001)));

        let v6 = parse_docker_proxy_cmdline(
            "/usr/bin/docker-proxy -proto tcp -host-ip :: -host-port 8888 -container-ip 172.17.0.3 -container-port 8080 -use-listen-fd",
        );
        assert_eq!(v6, Some((Protocol::Tcp, "::".to_string(), 8888)));

        let udp = parse_docker_proxy_cmdline(
            "/usr/bin/docker-proxy -proto udp -host-ip 127.0.0.1 -host-port 5353 -container-port 5353",
        );
        assert_eq!(udp, Some((Protocol::Udp, "127.0.0.1".to_string(), 5353)));
        assert_eq!(parse_docker_proxy_cmdline("/usr/bin/node server.js"), None);
    }
}
