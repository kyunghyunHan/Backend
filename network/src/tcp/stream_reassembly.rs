use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::{
    ethernet::{EtherTypes, EthernetPacket},
    ip::IpNextHeaderProtocols,
    ipv4::Ipv4Packet,
    tcp::TcpPacket,
    Packet,
};
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FlowKey {
    src_ip: String,
    src_port: u16,
    dst_ip: String,
    dst_port: u16,
}

#[derive(Debug)]
struct TcpStream {
    next_seq: u32,
    buffer: BTreeMap<u32, Vec<u8>>, // seq -> payload
}

impl TcpStream {
    fn new(seq: u32) -> Self {
        Self {
            next_seq: seq,
            buffer: BTreeMap::new(),
        }
    }

    fn push(&mut self, seq: u32, payload: &[u8]) {
        if payload.is_empty() {
            return;
        }

        self.buffer.insert(seq, payload.to_vec());

        // 가능한 만큼 이어붙여 출력
        while let Some(data) = self.buffer.remove(&self.next_seq) {
            print!("{}", String::from_utf8_lossy(&data));
            self.next_seq += data.len() as u32;
        }
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
        _ => return Err("unsupported channel".into()),
    };

    let mut streams: HashMap<FlowKey, TcpStream> = HashMap::new();

    loop {
        match rx.next() {
            Ok(packet) => {
                if let Some(eth) = EthernetPacket::new(packet) {
                    if eth.get_ethertype() == EtherTypes::Ipv4 {
                        if let Some(ipv4) = Ipv4Packet::new(eth.payload()) {
                            if ipv4.get_next_level_protocol() == IpNextHeaderProtocols::Tcp {
                                if let Some(tcp) = TcpPacket::new(ipv4.payload()) {
                                    let payload = tcp.payload();
                                    let seq = tcp.get_sequence();

                                    let key = FlowKey {
                                        src_ip: ipv4.get_source().to_string(),
                                        src_port: tcp.get_source(),
                                        dst_ip: ipv4.get_destination().to_string(),
                                        dst_port: tcp.get_destination(),
                                    };

                                    let stream = streams
                                        .entry(key)
                                        .or_insert_with(|| TcpStream::new(seq));

                                    stream.push(seq, payload);
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("read error: {}", e);
            }
        }
    }
}