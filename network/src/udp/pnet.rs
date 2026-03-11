use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::{
    ethernet::{EtherTypes, EthernetPacket},
    ip::IpNextHeaderProtocols,
    ipv4::Ipv4Packet,
    udp::UdpPacket,
    Packet,
};

fn hex_dump(data: &[u8]) {
    for (i, b) in data.iter().enumerate() {
        if i % 16 == 0 {
            print!("\n{:04x}: ", i);
        }
        print!("{:02x} ", b);
    }
    println!();
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
                // 1. raw bytes -> Ethernet frame
                if let Some(eth) = EthernetPacket::new(packet) {
                    // 2. Ethernet 안에 IPv4만 처리
                    if eth.get_ethertype() == EtherTypes::Ipv4 {
                        // 3. Ethernet payload -> IPv4 packet
                        if let Some(ipv4) = Ipv4Packet::new(eth.payload()) {
                            // 4. IPv4 안에 UDP만 처리
                            if ipv4.get_next_level_protocol() == IpNextHeaderProtocols::Udp {
                                // 5. IPv4 payload -> UDP packet
                                if let Some(udp) = UdpPacket::new(ipv4.payload()) {
                                    println!(
                                        "[UDP] {}:{} -> {}:{} | len={}",
                                        ipv4.get_source(),
                                        udp.get_source(),
                                        ipv4.get_destination(),
                                        udp.get_destination(),
                                        udp.get_length(),
                                    );

                                    let payload = udp.payload();
                                    if !payload.is_empty() {
                                        println!("  payload_len={}", payload.len());
                                        hex_dump(payload);
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