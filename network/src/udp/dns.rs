use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::{
    ethernet::{EtherTypes, EthernetPacket},
    ip::IpNextHeaderProtocols,
    ipv4::Ipv4Packet,
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

        // 압축 포인터는 이번 단계에서는 처리하지 않음
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
                            // 4. IPv4 안에 UDP만 처리
                            if ipv4.get_next_level_protocol() == IpNextHeaderProtocols::Udp {
                                // 5. IPv4 payload -> UDP
                                if let Some(udp) = UdpPacket::new(ipv4.payload()) {
                                    let src_port = udp.get_source();
                                    let dst_port = udp.get_destination();

                                    // 6. DNS는 보통 53 포트
                                    if src_port == 53 || dst_port == 53 {
                                        let payload = udp.payload();

                                        // DNS header는 최소 12바이트
                                        if payload.len() < 12 {
                                            continue;
                                        }

                                        let tx_id = match read_u16_be(payload, 0) {
                                            Some(v) => v,
                                            None => continue,
                                        };
                                        let flags = match read_u16_be(payload, 2) {
                                            Some(v) => v,
                                            None => continue,
                                        };
                                        let qdcount = match read_u16_be(payload, 4) {
                                            Some(v) => v,
                                            None => continue,
                                        };

                                        let qr = (flags & 0x8000) != 0;
                                        let kind = if qr { "RESPONSE" } else { "QUERY" };

                                        // 질문이 없으면 생략
                                        if qdcount == 0 {
                                            println!(
                                                "[DNS] {} {}:{} -> {}:{} tx_id=0x{:04x} qdcount=0",
                                                kind,
                                                ipv4.get_source(),
                                                src_port,
                                                ipv4.get_destination(),
                                                dst_port,
                                                tx_id
                                            );
                                            continue;
                                        }

                                        // 첫 질문은 DNS header(12바이트) 바로 뒤에 시작
                                        let (domain, next_offset) = match parse_dns_name(payload, 12) {
                                            Some(v) => v,
                                            None => {
                                                println!(
                                                    "[DNS] {} {}:{} -> {}:{} tx_id=0x{:04x} name_parse_failed",
                                                    kind,
                                                    ipv4.get_source(),
                                                    src_port,
                                                    ipv4.get_destination(),
                                                    dst_port,
                                                    tx_id
                                                );
                                                continue;
                                            }
                                        };

                                        let qtype = match read_u16_be(payload, next_offset) {
                                            Some(v) => v,
                                            None => continue,
                                        };

                                        let qclass = match read_u16_be(payload, next_offset + 2) {
                                            Some(v) => v,
                                            None => continue,
                                        };

                                        println!(
                                            "[DNS] {} {}:{} -> {}:{} tx_id=0x{:04x} domain={} type={} class={}",
                                            kind,
                                            ipv4.get_source(),
                                            src_port,
                                            ipv4.get_destination(),
                                            dst_port,
                                            tx_id,
                                            domain,
                                            dns_type_to_string(qtype),
                                            qclass
                                        );
                                    }
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