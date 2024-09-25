use tokio::net::UdpSocket;
use tokio::io;

#[tokio::main]
pub async fn main() -> io::Result<()> {
    // UDP 소켓을 바인딩 (127.0.0.1의 8080 포트)
    let socket = UdpSocket::bind("127.0.0.1:8080").await?;
    println!("Listening on 127.0.0.1:8080");

    let mut buf = vec![0u8; 1024];  // 1024 바이트 버퍼

    loop {
        // 클라이언트로부터 데이터 수신
        let (len, addr) = socket.recv_from(&mut buf).await?;

        println!("Received {} bytes from {}", len, addr);

        // 수신한 데이터를 그대로 클라이언트에게 전송 (에코)
        socket.send_to(&buf[..len], &addr).await?;

        println!("Echoed {} bytes to {}", len, addr);
    }
}
