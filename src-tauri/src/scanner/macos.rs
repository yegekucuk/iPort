use netstat2::{get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, TcpState};

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
    let Ok(sockets) = get_sockets_info(
        AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6,
        ProtocolFlags::TCP | ProtocolFlags::UDP,
    ) else {
        return Vec::new();
    };

    let mut out = Vec::new();
    for si in sockets {
        let pid = si.associated_pids.first().copied();
        match si.protocol_socket_info {
            ProtocolSocketInfo::Tcp(tcp) => {
                if tcp.state == TcpState::Listen && tcp.local_port > 0 {
                    out.push(RawSocket {
                        protocol: Protocol::Tcp,
                        port: tcp.local_port,
                        bind_address: tcp.local_addr.to_string(),
                        state: "LISTEN".to_string(),
                        pid,
                        uid: None,
                    });
                }
            }
            ProtocolSocketInfo::Udp(udp) => {
                if udp.local_port > 0 {
                    out.push(RawSocket {
                        protocol: Protocol::Udp,
                        port: udp.local_port,
                        bind_address: udp.local_addr.to_string(),
                        state: "UNCONN".to_string(),
                        pid,
                        uid: None,
                    });
                }
            }
        }
    }
    out
}
