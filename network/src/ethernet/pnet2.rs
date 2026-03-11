use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::{
    ethernet::{EtherTypes, EthernetPacket},
    ipv4::Ipv4Packet,
    Packet,
};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
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
                if let Some(eth) = EthernetPacket::new(packet) {
                    println!(
                        "[ETH] {} -> {} type={:?}",
                        eth.get_source(),
                        eth.get_destination(),
                        eth.get_ethertype()
                    );

                    match eth.get_ethertype() {
                        EtherTypes::Ipv4 => {
                            if let Some(ipv4) = Ipv4Packet::new(eth.payload()) {
                                println!(
                                    "[IPv4] {} -> {}",
                                    ipv4.get_source(),
                                    ipv4.get_destination()
                                );
                            } else {
                                println!("[IPv4] 파싱 실패");
                            }
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                eprintln!("[ERROR] failed to read packet: {}", e);
            }
        }
    }
}