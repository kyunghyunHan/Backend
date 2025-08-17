use network::{tcp::tcp_basic, udp::udp_basic};

fn main() {
    tcp_basic::example().unwrap();
    udp_basic::example().unwrap();
    
}
