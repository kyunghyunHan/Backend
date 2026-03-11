use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::{
    ethernet::{EtherTypes, EthernetPacket},
    ip::IpNextHeaderProtocols,
    ipv4::Ipv4Packet,
    tcp::TcpPacket,
    Packet,
};

fn tcp_flags_to_string(flags: u8) -> String {
    let mut parts = Vec::new();

    if flags & 0x01 != 0 {
        parts.push("FIN");
    }
    if flags & 0x02 != 0 {
        parts.push("SYN");
    }
    if flags & 0x04 != 0 {
        parts.push("RST");
    }
    if flags & 0x08 != 0 {
        parts.push("PSH");
    }
    if flags & 0x10 != 0 {
        parts.push("ACK");
    }
    if flags & 0x20 != 0 {
        parts.push("URG");
    }
    if flags & 0x40 != 0 {
        parts.push("ECE");
    }
    if flags & 0x80 != 0 {
        parts.push("CWR");
    }

    if parts.is_empty() {
        "NONE".to_string()
    } else {
        parts.join("|")
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let interface_name = "en0";

    let interfaces = datalink::interfaces();

    let interface = interfaces
        .into_iter()
        .find(|iface| iface.name == interface_name)
        .ok_or("인터페이스를 찾지 못했습니다.")?;

    println!("[INFO] listening on interface: {}", interface.name);

    let mut config = datalink::Config::default();
    config.read_timeout = None;

    let (_, mut rx) = match datalink::channel(&interface, config)? {
        Ethernet(tx, rx) => (tx, rx),
        _ => return Err("지원되지 않는 channel type".into()),
    };

    loop {
        match rx.next() {
            Ok(packet) => {
                // 1. Ethernet 프레임으로 해석
                if let Some(eth) = EthernetPacket::new(packet) {
                    // 2. Ethernet 안에 IPv4가 들어있는 경우만 처리
                    if eth.get_ethertype() == EtherTypes::Ipv4 {
                        // 3. Ethernet payload를 IPv4 패킷으로 해석
                        if let Some(ipv4) = Ipv4Packet::new(eth.payload()) {
                            // 4. IPv4 안에 TCP가 들어있는 경우만 처리
                            if ipv4.get_next_level_protocol() == IpNextHeaderProtocols::Tcp {
                                // 5. IPv4 payload를 TCP 패킷으로 해석
                                if let Some(tcp) = TcpPacket::new(ipv4.payload()) {
                                    println!(
                                        "[TCP] {}:{} -> {}:{} | flags={}",
                                        ipv4.get_source(),
                                        tcp.get_source(),
                                        ipv4.get_destination(),
                                        tcp.get_destination(),
                                        tcp_flags_to_string(tcp.get_flags()),
                                    );
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("[ERROR] failed to read packet: {}", e);
            }
        }
    }
}