// 학습용 축약판: DPDK 전체가 아니라 구조 이해용
// cargo add anyhow

use anyhow::{anyhow, Result};

const ETH_HEADER_LEN: usize = 14;
const IP_HEADER_MIN_LEN: usize = 20;
const TCP_HEADER_MIN_LEN: usize = 20;
const SOH: u8 = 0x01;

const ETHERTYPE_IPV4: u16 = 0x0800;
const IPPROTO_TCP: u8 = 6;

const TCP_FIN: u8 = 0x01;
const TCP_SYN: u8 = 0x02;
const TCP_RST: u8 = 0x04;
const TCP_PSH: u8 = 0x08;
const TCP_ACK: u8 = 0x10;

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
struct EthHeader {
    dst_mac: [u8; 6],
    src_mac: [u8; 6],
    ethertype: u16,
}

impl EthHeader {
    fn parse(data: &[u8]) -> Option<&Self> {
        if data.len() < ETH_HEADER_LEN {
            return None;
        }
        Some(unsafe { &*(data.as_ptr() as *const Self) })
    }

    fn ethertype(&self) -> u16 {
        u16::from_be(self.ethertype)
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
struct Ipv4Header {
    version_ihl: u8,
    dscp_ecn: u8,
    total_length: u16,
    identification: u16,
    flags_fragment: u16,
    ttl: u8,
    protocol: u8,
    checksum: u16,
    src_addr: u32,
    dst_addr: u32,
}

impl Ipv4Header {
    fn parse(data: &[u8]) -> Option<&Self> {
        if data.len() < IP_HEADER_MIN_LEN {
            return None;
        }
        Some(unsafe { &*(data.as_ptr() as *const Self) })
    }

    fn header_len(&self) -> usize {
        ((self.version_ihl & 0x0F) as usize) * 4
    }

    fn protocol(&self) -> u8 {
        self.protocol
    }

    fn src_addr_be(&self) -> u32 {
        u32::from_be(self.src_addr)
    }

    fn dst_addr_be(&self) -> u32 {
        u32::from_be(self.dst_addr)
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
struct TcpHeader {
    src_port: u16,
    dst_port: u16,
    seq_num: u32,
    ack_num: u32,
    data_offset_flags: u16,
    window: u16,
    checksum: u16,
    urgent_ptr: u16,
}

impl TcpHeader {
    fn parse(data: &[u8]) -> Option<&Self> {
        if data.len() < TCP_HEADER_MIN_LEN {
            return None;
        }
        Some(unsafe { &*(data.as_ptr() as *const Self) })
    }

    fn src_port(&self) -> u16 {
        u16::from_be(self.src_port)
    }

    fn dst_port(&self) -> u16 {
        u16::from_be(self.dst_port)
    }

    fn seq_num(&self) -> u32 {
        u32::from_be(self.seq_num)
    }

    fn ack_num(&self) -> u32 {
        u32::from_be(self.ack_num)
    }

    fn data_offset(&self) -> usize {
        ((u16::from_be(self.data_offset_flags) >> 12) as usize) * 4
    }

    fn flags(&self) -> u8 {
        (u16::from_be(self.data_offset_flags) & 0xFF) as u8
    }

    fn has_flag(&self, flag: u8) -> bool {
        self.flags() & flag != 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TcpState {
    Closed,
    SynSent,
    Established,
    FinWait1,
    FinWait2,
    TimeWait,
}

#[derive(Debug, Clone, Copy)]
struct ConnKey {
    local_addr: u32,
    local_port: u16,
    remote_addr: u32,
    remote_port: u16,
}

#[derive(Debug)]
struct TcpConnection {
    key: ConnKey,
    state: TcpState,
    snd_nxt: u32,
    snd_una: u32,
    rcv_nxt: u32,
}

#[derive(Debug)]
enum SegmentAction {
    Deliver,
    SendAck,
    SendDuplicateAck,
    ConnectionReset,
    Consumed,
    Drop,
}

impl TcpConnection {
    fn new(key: ConnKey) -> Self {
        Self {
            key,
            state: TcpState::SynSent,
            snd_nxt: 1,
            snd_una: 0,
            rcv_nxt: 0,
        }
    }

    fn on_segment(&mut self, tcp: &TcpHeader, payload: &[u8]) -> SegmentAction {
        match self.state {
            TcpState::SynSent => {
                if tcp.has_flag(TCP_SYN) && tcp.has_flag(TCP_ACK) {
                    self.rcv_nxt = tcp.seq_num().wrapping_add(1);
                    self.snd_una = tcp.ack_num();
                    self.state = TcpState::Established;
                    SegmentAction::SendAck
                } else {
                    SegmentAction::Drop
                }
            }
            TcpState::Established => {
                let seg_seq = tcp.seq_num();

                if tcp.has_flag(TCP_RST) {
                    self.state = TcpState::Closed;
                    return SegmentAction::ConnectionReset;
                }

                if tcp.has_flag(TCP_FIN) && seg_seq == self.rcv_nxt {
                    self.rcv_nxt = self.rcv_nxt.wrapping_add(1);
                    self.state = TcpState::TimeWait;
                    return SegmentAction::SendAck;
                }

                if seg_seq == self.rcv_nxt && tcp.has_flag(TCP_ACK) {
                    self.snd_una = tcp.ack_num();
                    if !payload.is_empty() {
                        self.rcv_nxt = self.rcv_nxt.wrapping_add(payload.len() as u32);
                        return SegmentAction::Deliver;
                    }
                    return SegmentAction::Consumed;
                }

                SegmentAction::SendDuplicateAck
            }
            _ => SegmentAction::Drop,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct FixField<'a> {
    tag: u32,
    value: &'a [u8],
}

struct FixParser<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> FixParser<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn next_field(&mut self) -> Option<FixField<'a>> {
        if self.pos >= self.data.len() {
            return None;
        }

        let mut tag = 0u32;
        while self.pos < self.data.len() && self.data[self.pos] != b'=' {
            let b = self.data[self.pos];
            if b.is_ascii_digit() {
                tag = tag * 10 + (b - b'0') as u32;
            }
            self.pos += 1;
        }

        if self.pos >= self.data.len() {
            return None;
        }

        self.pos += 1; // skip '='
        let value_start = self.pos;

        while self.pos < self.data.len() && self.data[self.pos] != SOH {
            self.pos += 1;
        }

        let value = &self.data[value_start..self.pos];

        if self.pos < self.data.len() {
            self.pos += 1; // skip SOH
        }

        Some(FixField { tag, value })
    }
}

#[derive(Debug)]
struct ExecReport<'a> {
    order_id: &'a [u8],
    cl_ord_id: &'a [u8],
    symbol: &'a [u8],
    exec_type: u8,
    seq_num: u32,
}

fn parse_uint(value: &[u8]) -> u32 {
    let mut result = 0u32;
    for &b in value {
        if b.is_ascii_digit() {
            result = result * 10 + (b - b'0') as u32;
        }
    }
    result
}

fn decode_exec_report(payload: &[u8]) -> Option<ExecReport<'_>> {
    let mut parser = FixParser::new(payload);

    let mut order_id = None;
    let mut cl_ord_id = None;
    let mut symbol = None;
    let mut exec_type = None;
    let mut seq_num = None;

    while let Some(field) = parser.next_field() {
        match field.tag {
            37 => order_id = Some(field.value),               // OrderID
            11 => cl_ord_id = Some(field.value),             // ClOrdID
            55 => symbol = Some(field.value),                // Symbol
            150 => exec_type = field.value.first().copied(), // ExecType
            34 => seq_num = Some(parse_uint(field.value)),   // MsgSeqNum
            10 => break,                                     // Checksum
            _ => {}
        }
    }

    Some(ExecReport {
        order_id: order_id?,
        cl_ord_id: cl_ord_id?,
        symbol: symbol?,
        exec_type: exec_type?,
        seq_num: seq_num?,
    })
}

fn process_packet(frame: &[u8], conn: &mut TcpConnection) -> Result<()> {
    let eth = EthHeader::parse(frame).ok_or_else(|| anyhow!("short ethernet frame"))?;
    if eth.ethertype() != ETHERTYPE_IPV4 {
        return Ok(());
    }

    let ip_data = &frame[ETH_HEADER_LEN..];
    let ip = Ipv4Header::parse(ip_data).ok_or_else(|| anyhow!("short ipv4 packet"))?;
    if ip.protocol() != IPPROTO_TCP {
        return Ok(());
    }

    let ip_hdr_len = ip.header_len();
    let tcp_data = &ip_data[ip_hdr_len..];
    let tcp = TcpHeader::parse(tcp_data).ok_or_else(|| anyhow!("short tcp packet"))?;
    let tcp_hdr_len = tcp.data_offset();
    let payload = &tcp_data[tcp_hdr_len..];

    println!(
        "TCP {}:{} -> {}:{} flags=0x{:02x} payload_len={}",
        ip.src_addr_be(),
        tcp.src_port(),
        ip.dst_addr_be(),
        tcp.dst_port(),
        tcp.flags(),
        payload.len()
    );

    match conn.on_segment(tcp, payload) {
        SegmentAction::Deliver => {
            if let Some(exec) = decode_exec_report(payload) {
                println!(
                    "FIX ExecReport order_id={} cl_ord_id={} symbol={} exec_type={} seq_num={}",
                    String::from_utf8_lossy(exec.order_id),
                    String::from_utf8_lossy(exec.cl_ord_id),
                    String::from_utf8_lossy(exec.symbol),
                    exec.exec_type as char,
                    exec.seq_num
                );
            } else {
                println!("FIX payload but decode failed");
            }
        }
        action => {
            println!("TCP action: {:?}", action);
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    // 실제 DPDK mbuf 대신 예시 payload만 보여주는 데모
    let fix_payload = b"8=FIX.4.2\x019=80\x0135=8\x0134=1274\x0137=ORDER123\x0111=CLO456\x01150=2\x0155=AAPL\x0110=178\x01";

    // 실제 네트워크 바이트를 만들진 않고, FIX parser만 간단 데모
    let exec = decode_exec_report(fix_payload).ok_or_else(|| anyhow!("fix decode failed"))?;
    println!(
        "demo FIX => order_id={} cl_ord_id={} symbol={} exec_type={} seq_num={}",
        String::from_utf8_lossy(exec.order_id),
        String::from_utf8_lossy(exec.cl_ord_id),
        String::from_utf8_lossy(exec.symbol),
        exec.exec_type as char,
        exec.seq_num
    );

    // process_packet()는 실제 Ethernet/IP/TCP frame 바이트가 있을 때 호출
    // ex) DPDK rx burst에서 받은 mbuf.data()
    Ok(())
}