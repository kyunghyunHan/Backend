use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::{
    ethernet::{EtherTypes, EthernetPacket},
    ip::IpNextHeaderProtocols,
    ipv4::Ipv4Packet,
    tcp::TcpPacket,
    udp::UdpPacket,
    Packet,
};
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Proto {
    Tcp,
    Udp,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FlowKey {
    src_ip: String,
    src_port: u16,
    dst_ip: String,
    dst_port: u16,
    proto: Proto,
}

#[derive(Debug, Clone)]
struct FlowStats {
    packets: u64,
    bytes: u64,
    first_seen: Instant,
    last_seen: Instant,
}

impl FlowStats {
    fn new(now: Instant, bytes: usize) -> Self {
        Self {
            packets: 1,
            bytes: bytes as u64,
            first_seen: now,
            last_seen: now,
        }
    }

    fn update(&mut self, now: Instant, bytes: usize) {
        self.packets += 1;
        self.bytes += bytes as u64;
        self.last_seen = now;
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

    let mut flows: HashMap<FlowKey, FlowStats> = HashMap::new();
    let mut last_report = Instant::now();

    loop {
        match rx.next() {
            Ok(packet) => {
                if let Some(eth) = EthernetPacket::new(packet) {
                    if eth.get_ethertype() == EtherTypes::Ipv4 {
                        if let Some(ipv4) = Ipv4Packet::new(eth.payload()) {
                            match ipv4.get_next_level_protocol() {
                                IpNextHeaderProtocols::Tcp => {
                                    if let Some(tcp) = TcpPacket::new(ipv4.payload()) {
                                        let key = FlowKey {
                                            src_ip: ipv4.get_source().to_string(),
                                            src_port: tcp.get_source(),
                                            dst_ip: ipv4.get_destination().to_string(),
                                            dst_port: tcp.get_destination(),
                                            proto: Proto::Tcp,
                                        };

                                        let now = Instant::now();
                                        let entry = flows
                                            .entry(key)
                                            .or_insert_with(|| FlowStats::new(now, packet.len()));
                                        entry.update(now, packet.len());
                                    }
                                }
                                IpNextHeaderProtocols::Udp => {
                                    if let Some(udp) = UdpPacket::new(ipv4.payload()) {
                                        let key = FlowKey {
                                            src_ip: ipv4.get_source().to_string(),
                                            src_port: udp.get_source(),
                                            dst_ip: ipv4.get_destination().to_string(),
                                            dst_port: udp.get_destination(),
                                            proto: Proto::Udp,
                                        };

                                        let now = Instant::now();
                                        let entry = flows
                                            .entry(key)
                                            .or_insert_with(|| FlowStats::new(now, packet.len()));
                                        entry.update(now, packet.len());
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }

                // 5초마다 flow 요약 출력
                if last_report.elapsed() >= Duration::from_secs(5) {
                    print_flows(&flows);
                    remove_stale_flows(&mut flows, Duration::from_secs(30));
                    last_report = Instant::now();
                }
            }
            Err(e) => {
                eprintln!("[ERROR] failed to read packet: {}", e);
            }
        }
    }
}

fn print_flows(flows: &HashMap<FlowKey, FlowStats>) {
    println!();
    println!("================ FLOW TABLE ================");

    let mut items: Vec<(&FlowKey, &FlowStats)> = flows.iter().collect();

    // bytes 기준 내림차순
    items.sort_by(|a, b| b.1.bytes.cmp(&a.1.bytes));

    for (key, stats) in items.iter().take(20) {
        let proto = match key.proto {
            Proto::Tcp => "TCP",
            Proto::Udp => "UDP",
        };

        let age_secs = stats.first_seen.elapsed().as_secs();
        let idle_secs = stats.last_seen.elapsed().as_secs();

        println!(
            "[{}] {}:{} -> {}:{} | packets={} bytes={} age={}s idle={}s",
            proto,
            key.src_ip,
            key.src_port,
            key.dst_ip,
            key.dst_port,
            stats.packets,
            stats.bytes,
            age_secs,
            idle_secs
        );
    }

    println!("total flows: {}", flows.len());
    println!("============================================");
    println!();
}

fn remove_stale_flows(flows: &mut HashMap<FlowKey, FlowStats>, timeout: Duration) {
    flows.retain(|_, stats| stats.last_seen.elapsed() < timeout);
}