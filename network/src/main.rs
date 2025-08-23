use network::{
    bloking::basic as bloking, grpc::basic as grpc, quic::basic as quic, simd::basic as simd,
    tcp::tcp_basic, udp::udp_basic,
};

fn main() {
    // tcp_basic::example().unwrap();
    // udp_basic::example().unwrap();
    // bloking::nonblocking_way();
    // quic::example().unwrap();
    // simd::example();
    grpc::example();
}
