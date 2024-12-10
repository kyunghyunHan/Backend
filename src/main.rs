use axum_socket::{tcp_example, udp_example,websocket,rabbit_mq};

fn main(){
    rabbit_mq::main().unwrap();
    // websocket::example();
    // udp_example::example();
    
}