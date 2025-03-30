use axum_socket::{multicast, rabbit_mq, tcp, tcp_example, udp_example, websocket};
use network_study::implementation::tcp;

fn main() {
    // rabbit_mq::main().unwrap();
    // websocket::example();
    // udp_example::example();
    // multicast::example();
    tcp::main();
}
