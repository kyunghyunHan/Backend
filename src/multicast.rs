use std::io;
use std::net::UdpSocket;

pub fn example() -> io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0.:0")?;
    //TTL설정
    socket.set_multicast_ttl_v4(5)?;//최대 5개의 라우터 까지만 전달


    // socket.set_multicast_ttl_v4(1)?;  // 같은 서브넷으로만 제한
    
    // // 조직 내부로 제한
    // socket.set_multicast_ttl_v4(32)?; // 같은 사이트로 제한
    
    // // 글로벌 전송 허용
    // socket.set_multicast_ttl_v4(255)?; // 제한 없음
    Ok(())
}
