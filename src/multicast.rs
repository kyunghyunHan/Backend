use std::io;
use std::net::UdpSocket;

pub fn example() -> io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0.:0")?;
    //TTL설정
    socket.set_multicast_ttl_v4(5)?;

    Ok(())
}
