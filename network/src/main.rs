use network::implementation::tcp;
use network::{multicast, rabbit_mq, tcp_example, udp_example, websocket};

fn main() {
    // rabbit_mq::main().unwrap();
    // websocket::example();
    // udp_example::example();
    // multicast::example();
    tcp::main();
}
