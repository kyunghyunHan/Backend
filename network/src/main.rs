use futures_util::TryFutureExt;
use network::{
    bloking::basic as bloking, graph_ql::basic as graph_ql, grpc::basic as grpc, numa,
    quic::basic as quic, simd::basic as simd, tcp::tcp_basic, udp::udp_basic,
};

fn main() {
    // tcp_basic::example().unwrap();
    // udp_basic::example().unwrap();
    // bloking::nonblocking_way();
    // quic::example().unwrap();
    // simd::example();
    // grpc::example();
    // graph_ql::example().unwrap_or_else(|x|{
    //     println!("{}","error")
    // });
    numa::example();
}
