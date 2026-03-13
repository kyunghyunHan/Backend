use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::{
    ethernet::{EtherTypes, EthernetPacket},
    ip::IpNextHeaderProtocols,
    ipv4::Ipv4Packet,
    tcp::TcpPacket,
    udp::UdpPacket,
    Packet,
};

fn read_u16_be(data: &[u8], offset: usize) -> Option<u16> {
    if offset + 1 >= data.len() {
        return None;
    }
    Some(u16::from_be_bytes([data[offset], data[offset + 1]]))
}

fn parse_dns_name(data: &[u8], mut offset: usize) -> Option<(String, usize)> {
    let mut labels = Vec::new();

    loop {
        if offset >= data.len() {
            return None;
        }

        let len = data[offset] as usize;
        offset += 1;

        if len == 0 {
            break;
        }

        // 이번 단계에서는 압축 포인터 미지원
        if len & 0b1100_0000 != 0 {
            return None;
        }

        if offset + len > data.len() {
            return None;
        }

        let label = std::str::from_utf8(&data[offset..offset + len]).ok()?;
        labels.push(label.to_string());
        offset += len;
    }

    Some((labels.join("."), offset))
}

fn dns_type_to_string(qtype: u16) -> &'static str {
    match qtype {
        1 => "A",
        2 => "NS",
        5 => "CNAME",
        6 => "SOA",
        12 => "PTR",
        15 => "MX",
        16 => "TXT",
        28 => "AAAA",
        _ => "OTHER",
    }
}

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

fn handle_dns(payload: &[u8]) -> Option<String> {
    if payload.len() < 12 {
        return None;
    }

    let tx_id = read_u16_be(payload, 0)?;
    let flags = read_u16_be(payload, 2)?;
    let qdcount = read_u16_be(payload, 4)?;

    let qr = (flags & 0x8000) != 0;
    let kind = if qr { "RESPONSE" } else { "QUERY" };

    if qdcount == 0 {
        return Some(format!(
            "[DNS] kind={} tx_id=0x{:04x} qdcount=0",
            kind, tx_id
        ));
    }

    let (domain, next_offset) = parse_dns_name(payload, 12)?;
    let qtype = read_u16_be(payload, next_offset)?;
    let qclass = read_u16_be(payload, next_offset + 2)?;

    Some(format!(
        "[DNS] kind={} tx_id=0x{:04x} domain={} type={} class={}",
        kind,
        tx_id,
        domain,
        dns_type_to_string(qtype),
        qclass
    ))
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
                // 1. raw bytes -> Ethernet
                if let Some(eth) = EthernetPacket::new(packet) {
                    // 2. Ethernet 안에 IPv4만 처리
                    if eth.get_ethertype() == EtherTypes::Ipv4 {
                        // 3. Ethernet payload -> IPv4
                        if let Some(ipv4) = Ipv4Packet::new(eth.payload()) {
                            match ipv4.get_next_level_protocol() {
                                IpNextHeaderProtocols::Tcp => {
                                    // 4-A. IPv4 payload -> TCP
                                    if let Some(tcp) = TcpPacket::new(ipv4.payload()) {
                                        println!(
                                            "[TCP] {}:{} -> {}:{} | flags={} | seq={} | ack={}",
                                            ipv4.get_source(),
                                            tcp.get_source(),
                                            ipv4.get_destination(),
                                            tcp.get_destination(),
                                            tcp_flags_to_string(tcp.get_flags()),
                                            tcp.get_sequence(),
                                            tcp.get_acknowledgement(),
                                        );
                                    }
                                }
                                IpNextHeaderProtocols::Udp => {
                                    // 4-B. IPv4 payload -> UDP
                                    if let Some(udp) = UdpPacket::new(ipv4.payload()) {
                                        let src_port = udp.get_source();
                                        let dst_port = udp.get_destination();

                                        println!(
                                            "[UDP] {}:{} -> {}:{} | len={}",
                                            ipv4.get_source(),
                                            src_port,
                                            ipv4.get_destination(),
                                            dst_port,
                                            udp.get_length(),
                                        );

                                        // 5. DNS(53)면 추가 파싱
                                        if src_port == 53 || dst_port == 53 {
                                            if let Some(dns_info) = handle_dns(udp.payload()) {
                                                println!("  {}", dns_info);
                                            }
                                        }
                                    }
                                }
                                _ => {}
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