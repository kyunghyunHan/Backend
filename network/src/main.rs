use network::{tcp::tcp_basic, udp::udp_basic,bloking::basic as bloking,quic::basic as quic};


fn main() {
    // tcp_basic::example().unwrap();
    // udp_basic::example().unwrap();
    // bloking::nonblocking_way();
    quic::example().unwrap();
}
